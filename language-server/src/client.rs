use crate::jsonrpc::*;
use async_trait::async_trait;
use chashmap::CHashMap;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use language_server_derive::*;
use lsp_types::*;
use serde::Serialize;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

/// Defines the client-side implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
#[jsonrpc_client(ident = "LanguageClient")]
#[async_trait]
trait LanguageClient {
    /// The base protocol offers also support to report progress in a generic fashion.
    /// [This mechanism](https://microsoft.github.io/language-server-protocol/specification#progress)
    /// can be used to report any kind of progress including work done progress
    /// (usually used to report progress in the user interface using a progress bar)
    /// and partial result progress to support streaming of results.
    #[jsonrpc_method("$/progress", kind = "notification")]
    async fn progress(&self, params: ProgressParams);

    /// The [show message notification](https://microsoft.github.io/language-server-protocol/specification#window_showMessage)
    /// is sent from a server to a client to ask the client to display a particular message in the user interface.
    #[jsonrpc_method("window/showMessage", kind = "notification")]
    async fn show_message(&self, params: ShowMessageParams);

    /// The [show message request](https://microsoft.github.io/language-server-protocol/specification#window_showMessageRequest)
    /// is sent from a server to a client to ask the client to display a particular message in the user interface.
    #[jsonrpc_method("window/showMessageRequest", kind = "request")]
    async fn show_message_request(
        &self,
        params: ShowMessageRequestParams,
    ) -> Result<Option<MessageActionItem>>;

    /// The [log message notification](https://microsoft.github.io/language-server-protocol/specification#window_logMessage)
    /// is sent from the server to the client to ask the client to log a particular message.
    #[jsonrpc_method("window/logMessage", kind = "notification")]
    async fn log_message(&self, params: LogMessageParams);

    /// The [`window/workDoneProgress/create`](https://microsoft.github.io/language-server-protocol/specification#window_workDoneProgress_create)
    /// request is sent from the server to the client to ask the client to create a work done progress.
    #[jsonrpc_method("window/workDoneProgress/create", kind = "request")]
    async fn work_done_progress_create(&self, params: WorkDoneProgressCreateParams) -> Result<()>;

    /// The [telemetry notification](https://microsoft.github.io/language-server-protocol/specification#telemetry_event)
    /// is sent from the server to the client to ask the client to log a telemetry event.
    #[jsonrpc_method("telemetry/event", kind = "notification")]
    async fn telemetry_event(&self, params: serde_json::Value);

    /// The [`client/registerCapability`](https://microsoft.github.io/language-server-protocol/specification#client_registerCapability)
    /// request is sent from the server to the client to register for a new capability on the client side.
    #[jsonrpc_method("client/registerCapability", kind = "request")]
    async fn register_capability(&self, params: RegistrationParams) -> Result<()>;

    /// The [`client/unregisterCapability`](https://microsoft.github.io/language-server-protocol/specification#client_unregisterCapability)
    /// request is sent from the server to the client to unregister a previously registered capability.
    #[jsonrpc_method("client/unregisterCapability", kind = "request")]
    async fn unregister_capability(&self, params: UnregistrationParams) -> Result<()>;

    /// The [`workspace/workspaceFolders`](https://microsoft.github.io/language-server-protocol/specification#workspace_workspaceFolders)
    /// request is sent from the server to the client to fetch the current open list of workspace folders.
    #[jsonrpc_method("workspace/workspaceFolders", kind = "request")]
    async fn workspace_folders(&self, params: ()) -> Result<Vec<WorkspaceFolder>>;

    /// The [`workspace/configuration`](https://microsoft.github.io/language-server-protocol/specification#workspace_configuration)
    /// request is sent from the server to the client to fetch configuration settings from the client.
    #[jsonrpc_method("workspace/configuration", kind = "request")]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value>;

    /// The [`workspace/applyEdit`](https://microsoft.github.io/language-server-protocol/specification#workspace_applyEdit)
    /// request is sent from the server to the client to modify resource on the client side.
    #[jsonrpc_method("workspace/applyEdit", kind = "request")]
    async fn apply_edit(
        &self,
        params: ApplyWorkspaceEditParams,
    ) -> Result<ApplyWorkspaceEditResponse>;

    /// [Diagnostics notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_publishDiagnostics)
    /// are sent from the server to the client to signal results of validation runs.
    #[jsonrpc_method("textDocument/publishDiagnostics", kind = "notification")]
    async fn publish_diagnostics(&self, params: PublishDiagnosticsParams);
}

#[async_trait]
pub trait ResponseHandler {
    async fn handle(&self, response: Response);
}

#[derive(Debug)]
pub struct Client {
    output: mpsc::Sender<String>,
    request_id: AtomicU64,
    senders_by_id: CHashMap<Id, oneshot::Sender<Result<serde_json::Value>>>,
}

impl Client {
    pub fn new(output: mpsc::Sender<String>) -> Self {
        Self {
            output,
            request_id: AtomicU64::new(0),
            senders_by_id: CHashMap::new(),
        }
    }

    pub async fn send_request<T: Serialize>(
        &self,
        method: String,
        params: T,
    ) -> Result<serde_json::Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = Request::new(method, json!(params), Id::Number(id));

        let (result_tx, result_rx) = oneshot::channel();
        self.senders_by_id.insert(request.id.clone(), result_tx);
        self.send(Message::Request(request)).await;

        result_rx.await.unwrap()
    }

    pub async fn send_notification<T: Serialize>(&self, method: String, params: T) {
        let notification = Notification::new(method, json!(params));
        self.send(Message::Notification(notification)).await;
    }

    async fn send(&self, message: Message) {
        let mut output = self.output.clone();
        let json = serde_json::to_string(&message).unwrap();
        output.send(json).await.unwrap();
    }
}

#[async_trait]
impl ResponseHandler for Client {
    async fn handle(&self, response: Response) {
        let id = response.id.expect("Expected response with id");
        let result = match response.error {
            Some(why) => Err(why),
            None => Ok(response.result.unwrap_or(serde_json::Value::Null)),
        };

        let result_tx = self
            .senders_by_id
            .remove(&id)
            .expect("Unexpected response received");
        result_tx.send(result).unwrap();
    }
}
