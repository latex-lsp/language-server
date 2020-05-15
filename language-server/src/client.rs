use crate::jsonrpc::*;
use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    lock::Mutex,
    prelude::*,
};
use language_server_macros::*;
use lsp_types::*;
use serde::Serialize;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

/// Defines the client-side implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
#[jsonrpc_client(ident = "LanguageClientImpl")]
#[async_trait]
pub trait LanguageClient: Sync {
    /// The base protocol offers also support to report progress in a generic fashion.
    /// [This mechanism](https://microsoft.github.io/language-server-protocol/specification#progress)
    /// can be used to report any kind of progress including work done progress
    /// (usually used to report progress in the user interface using a progress bar)
    /// and partial result progress to support streaming of results.
    #[jsonrpc_method(name = "$/progress", kind = "notification")]
    async fn progress(&self, params: ProgressParams);

    /// The [show message notification](https://microsoft.github.io/language-server-protocol/specification#window_showMessage)
    /// is sent from a server to a client to ask the client to display a particular message in the user interface.
    #[jsonrpc_method(name = "window/showMessage", kind = "notification")]
    async fn show_message(&self, params: ShowMessageParams);

    /// The [show message request](https://microsoft.github.io/language-server-protocol/specification#window_showMessageRequest)
    /// is sent from a server to a client to ask the client to display a particular message in the user interface.
    #[jsonrpc_method(name = "window/showMessageRequest", kind = "request")]
    async fn show_message_request(
        &self,
        params: ShowMessageRequestParams,
    ) -> Result<Option<MessageActionItem>>;

    /// The [log message notification](https://microsoft.github.io/language-server-protocol/specification#window_logMessage)
    /// is sent from the server to the client to ask the client to log a particular message.
    #[jsonrpc_method(name = "window/logMessage", kind = "notification")]
    async fn log_message(&self, params: LogMessageParams);

    /// The [`window/workDoneProgress/create`](https://microsoft.github.io/language-server-protocol/specification#window_workDoneProgress_create)
    /// request is sent from the server to the client to ask the client to create a work done progress.
    #[jsonrpc_method(name = "window/workDoneProgress/create", kind = "request")]
    async fn work_done_progress_create(&self, params: WorkDoneProgressCreateParams) -> Result<()>;

    /// The [telemetry notification](https://microsoft.github.io/language-server-protocol/specification#telemetry_event)
    /// is sent from the server to the client to ask the client to log a telemetry event.
    #[jsonrpc_method(name = "telemetry/event", kind = "notification")]
    async fn telemetry_event(&self, params: serde_json::Value);

    /// The [`client/registerCapability`](https://microsoft.github.io/language-server-protocol/specification#client_registerCapability)
    /// request is sent from the server to the client to register for a new capability on the client side.
    #[jsonrpc_method(name = "client/registerCapability", kind = "request")]
    async fn register_capability(&self, params: RegistrationParams) -> Result<()>;

    /// The [`client/unregisterCapability`](https://microsoft.github.io/language-server-protocol/specification#client_unregisterCapability)
    /// request is sent from the server to the client to unregister a previously registered capability.
    #[jsonrpc_method(name = "client/unregisterCapability", kind = "request")]
    async fn unregister_capability(&self, params: UnregistrationParams) -> Result<()>;

    /// The [`workspace/workspaceFolders`](https://microsoft.github.io/language-server-protocol/specification#workspace_workspaceFolders)
    /// request is sent from the server to the client to fetch the current open list of workspace folders.
    #[jsonrpc_method(name = "workspace/workspaceFolders", kind = "request")]
    async fn workspace_folders(&self, params: ()) -> Result<Vec<WorkspaceFolder>>;

    /// The [`workspace/configuration`](https://microsoft.github.io/language-server-protocol/specification#workspace_configuration)
    /// request is sent from the server to the client to fetch configuration settings from the client.
    #[jsonrpc_method(name = "workspace/configuration", kind = "request")]
    async fn configuration(&self, params: ConfigurationParams) -> Result<serde_json::Value>;

    /// The [`workspace/applyEdit`](https://microsoft.github.io/language-server-protocol/specification#workspace_applyEdit)
    /// request is sent from the server to the client to modify resource on the client side.
    #[jsonrpc_method(name = "workspace/applyEdit", kind = "request")]
    async fn apply_edit(
        &self,
        params: ApplyWorkspaceEditParams,
    ) -> Result<ApplyWorkspaceEditResponse>;

    /// [Diagnostics notification](https://microsoft.github.io/language-server-protocol/specification#textDocument_publishDiagnostics)
    /// are sent from the server to the client to signal results of validation runs.
    #[jsonrpc_method(name = "textDocument/publishDiagnostics", kind = "notification")]
    async fn publish_diagnostics(&self, params: PublishDiagnosticsParams);

    /// The `textDocument/semanticHighlighting` notification is pushed from the server to the client
    /// to inform the client about additional semantic highlighting information that has to be applied on the text document.
    #[cfg_attr(docsrs, doc(cfg(feature = "proposed")))]
    #[cfg(feature = "proposed")]
    #[jsonrpc_method(name = "textDocument/semanticHighlighting", kind = "notification")]
    async fn semantic_highlighting(&self, params: SemanticHighlightingParams);
}

#[async_trait]
pub trait ResponseHandler {
    async fn handle(&self, response: Response);
}

#[derive(Debug)]
pub struct Client {
    output: mpsc::Sender<Message>,
    request_id: AtomicU64,
    senders_by_id: Mutex<HashMap<Id, oneshot::Sender<Result<serde_json::Value>>>>,
}

impl Client {
    pub fn new(output: mpsc::Sender<Message>) -> Self {
        Self {
            output,
            request_id: AtomicU64::new(0),
            senders_by_id: Mutex::new(HashMap::new()),
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
        {
            let mut senders_by_id = self.senders_by_id.lock().await;
            senders_by_id.insert(request.id.clone(), result_tx);
        }

        let mut output = self.output.clone();
        output.send(Message::Request(request)).await.unwrap();

        result_rx.await.unwrap()
    }

    pub async fn send_notification<T: Serialize>(&self, method: String, params: T) {
        let notification = Notification::new(method, json!(params));
        let mut output = self.output.clone();
        output
            .send(Message::Notification(notification))
            .await
            .unwrap();
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

        let result_tx = {
            let mut senders_by_id = self.senders_by_id.lock().await;
            senders_by_id
                .remove(&id)
                .expect("Unexpected response received")
        };

        result_tx.send(result).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::{join, join3};

    #[tokio::test]
    async fn notification() {
        let (tx, mut rx) = mpsc::channel(0);
        let client = Client::new(tx);
        let ((), output) = join(client.send_notification("foo".into(), 42u64), rx.next()).await;

        assert_eq!(
            output.unwrap(),
            Message::Notification(Notification::new("foo".to_owned(), json!(42)))
        );
    }

    #[tokio::test]
    async fn request_success() {
        let (tx, mut rx) = mpsc::channel(0);
        let client = Client::new(tx);
        let (response, output, ()) = join3(
            client.send_request("foo".into(), 42u64),
            rx.next(),
            client.handle(Response::result(
                serde_json::to_value(1337u64).unwrap(),
                Id::Number(0),
            )),
        )
        .await;
        assert_eq!(
            output.unwrap(),
            Message::Request(Request::new("foo".to_owned(), json!(42), Id::Number(0)))
        );
        assert_eq!(
            serde_json::from_value::<u64>(response.unwrap()).unwrap(),
            1337
        );
    }

    #[tokio::test]
    async fn request_failure() {
        let (tx, mut rx) = mpsc::channel(0);
        let client = Client::new(tx);
        let (response, output, ()) = join3(
            client.send_request("foo".into(), 42u64),
            rx.next(),
            client.handle(Response::error(
                Error::internal_error("bar".into()),
                Some(Id::Number(0)),
            )),
        )
        .await;
        assert_eq!(
            output.unwrap(),
            Message::Request(Request::new("foo".to_owned(), json!(42), Id::Number(0)))
        );
        assert_eq!(response.unwrap_err(), Error::internal_error("bar".into()));
    }

    #[tokio::test]
    #[should_panic(expected = "Unexpected response received")]
    async fn request_unexpected_response() {
        let (tx, _) = mpsc::channel(0);
        let client = Client::new(tx);
        client
            .handle(Response::error(
                Error::internal_error("bar".into()),
                Some(Id::Number(42)),
            ))
            .await;
    }

    #[tokio::test]
    #[should_panic(expected = "Expected response with id")]
    async fn request_response_without_id() {
        let (tx, _) = mpsc::channel(0);
        let client = Client::new(tx);
        client
            .handle(Response::error(Error::internal_error("bar".into()), None))
            .await;
    }
}
