mod client;
mod codec;
pub mod jsonrpc;
mod server;

pub use client::{LanguageClient, LspClient};
pub use server::LanguageServer;

use client::ResponseHandler;
use codec::LspCodec;
use futures::{channel::mpsc, prelude::*, sink::SinkExt, stream::StreamExt};
use jsonrpc::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::prelude::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};

macro_rules! handle {
    ($handle:path, $server:expr, $client:expr, $msg:expr, { $($method:pat => $handler:ident),* }, $default:expr) => {{
        match $msg.method.as_ref() {
            $(
                $method => $handle($msg, |p, c| $server.$handler(p, c), $client).await,
            )*
            _ => $default,
        }
    }};
}

/// Represents a service that processes messages according to the
/// [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
pub struct LspService<I, O, S> {
    input: I,
    output: O,
    output_tx: mpsc::Sender<String>,
    output_rx: mpsc::Receiver<String>,
    server: Arc<S>,
    client: LspClient,
}

impl<I, O, S> LspService<I, O, S>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Send + Unpin + 'static,
    S: LanguageServer + Send + Sync + 'static,
{
    /// Creates a new `LspService`.
    pub fn new(input: I, output: O, server: Arc<S>) -> Self {
        let (output_tx, output_rx) = mpsc::channel(0);
        let client = LspClient::new(output_tx.clone());

        Self {
            input,
            output,
            output_tx,
            output_rx,
            server,
            client,
        }
    }

    /// Starts the service and processes messages.
    /// It is guaranteed that all notifications are processed in order.
    pub async fn listen(self) {
        let output = self.output;
        let mut output_rx = self.output_rx;
        tokio::spawn(async move {
            let mut output = FramedWrite::new(output, LspCodec);
            loop {
                let message = output_rx.next().await.unwrap();
                output.send(message).await.unwrap();
            }
        });

        let mut input = FramedRead::new(self.input, LspCodec);
        while let Some(Ok(json)) = input.next().await {
            Self::handle_incoming(
                Arc::clone(&self.server),
                self.client.clone(),
                self.output_tx.clone(),
                &json,
            )
            .await;
        }
    }

    async fn handle_incoming(
        server: Arc<S>,
        client: LspClient,
        mut output: mpsc::Sender<String>,
        json: &str,
    ) {
        match serde_json::from_str(json).map_err(|_| Error::parse_error()) {
            Ok(Message::Request(request)) => {
                let server = Arc::clone(&server);
                let client = client.clone();
                tokio::spawn(async move {
                    let response = handle!(Self::handle_request, server, client, request, {
                            "initialize" => initialize,
                            "shutdown" => shutdown,
                            "workspace/symbol" => workspace_symbol,
                            "workspace/executeCommand" => execute_command,
                            "textDocument/willSaveWaitUntil" => will_save_wait_until,
                            "textDocument/completion" => completion,
                            "completionItem/resolve" => completion_resolve,
                            "textDocument/hover" => hover,
                            "textDocument/signatureHelp" => signature_help,
                            "textDocument/declaration" => declaration,
                            "textDocument/definition" => definition,
                            "textDocument/typeDefinition" => type_definition,
                            "textDocument/implementation" => implementation,
                            "textDocument/references" => references,
                            "textDocument/documentHighlight" => document_highlight,
                            "textDocument/documentSymbol" => document_symbol,
                            "textDocument/codeAction" => code_action,
                            "textDocument/codeLens" => code_lens,
                            "codeLens/resolve" => code_lens_resolve,
                            "textDocument/documentLink" => document_link,
                            "documentLink/resolve" => document_link_resolve,
                            "textDocument/documentColor" => document_color,
                            "textDocument/colorPresentation" => color_presentation,
                            "textDocument/formatting" => formatting,
                            "textDocument/rangeFormatting" => range_formatting,
                            "textDocument/onTypeFormatting" => on_type_formatting,
                            "textDocument/rename" => rename,
                            "textDocument/prepareRename" => prepare_rename,
                            "textDocument/foldingRange" => folding_range,
                            "textDocument/selectionRange" => selection_range
                        },
                        Response::error(Error::method_not_found_error(), Some(request.id))
                    );

                    let json = serde_json::to_string(&response).unwrap();
                    output.send(json).await.unwrap();
                });
            }
            Ok(Message::Notification(notification)) => {
                handle!(Self::handle_notification, server, client, notification, {
                    "initialized" => initialized,
                    "exit" => exit,
                    "window/workDoneProgress/cancel" => work_done_progress_cancel,
                    "workspace/didChangeWorkspaceFolders" => did_change_workspace_folders,
                    "workspace/didChangeConfiguration" => did_change_configuration,
                    "workspace/didChangeWatchedFiles" => did_change_watched_files,
                    "textDocument/didOpen" => did_open,
                    "textDocument/didChange" => did_change,
                    "textDocument/willSave" => will_save,
                    "textDocument/didSave" => did_save,
                    "textDocument/didClose" => did_close
                }, ());
            }
            Ok(Message::Response(response)) => {
                client.handle(response).await;
            }
            Err(why) => {
                let response = Response::error(why, None);
                let json = serde_json::to_string(&response).unwrap();
                output.send(json).await.unwrap();
            }
        };
    }

    async fn handle_request<'a, H, F, P, R>(
        request: Request,
        handler: H,
        client: LspClient,
    ) -> Response
    where
        H: Fn(P, LspClient) -> F + Send + Sync + 'a,
        F: Future<Output = server::Result<R>> + Send,
        P: DeserializeOwned + Send,
        R: Serialize,
    {
        let handle = |json| async move {
            let params = serde_json::from_value(json).map_err(|_| Error::deserialize_error())?;
            let result = handler(params, client)
                .await
                .map_err(Error::internal_error)?;
            Ok(result)
        };

        match handle(request.params).await {
            Ok(result) => Response::result(json!(result), request.id),
            Err(error) => Response::error(error, Some(request.id)),
        }
    }

    async fn handle_notification<'a, H, F, P>(
        notification: Notification,
        handler: H,
        client: LspClient,
    ) where
        H: Fn(P, LspClient) -> F + Send + Sync + 'a,
        F: Future<Output = ()> + Send,
        P: DeserializeOwned + Send,
    {
        let error = Error::deserialize_error().message;
        let params = serde_json::from_value(notification.params).expect(&error);
        handler(params, client).await;
    }
}
