mod codec;
pub mod jsonrpc;
mod server;

pub use server::LanguageServer;

use codec::LspCodec;
use futures::{channel::mpsc, prelude::*, sink::SinkExt, stream::StreamExt};
use jsonrpc::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::prelude::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};

macro_rules! handle {
    ($handle:path, $server:expr, $msg:expr, { $($method:pat => $handler:ident),* }, $default:expr) => {{
        match $msg.method.as_ref() {
            $(
                $method => $handle($msg, |p| $server.$handler(p)).await,
            )*
            _ => $default,
        }
    }};
}

/// Represents a service that processes messages according to the
/// [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
#[derive(Debug)]
pub struct LspService<S> {
    server: Arc<S>,
}

impl<S> LspService<S>
where
    S: LanguageServer + Send + Sync + 'static,
{
    /// Creates a new `LspService`.
    pub fn new(server: Arc<S>) -> Self {
        Self { server }
    }

    /// Starts the service and process message from the given `input` to the given `output`.
    ///
    /// It is guaranteed that all notifications are processed in order.
    pub async fn listen<I, O>(&self, input: I, output: O)
    where
        I: AsyncRead + Unpin,
        O: AsyncWrite + Send + Unpin + 'static,
    {
        let mut input = FramedRead::new(input, LspCodec);
        let (output_tx, mut output_rx) = mpsc::channel(0);

        tokio::spawn(async move {
            let mut output = FramedWrite::new(output, LspCodec);
            loop {
                let message = output_rx.next().await.unwrap();
                output.send(message).await.unwrap();
            }
        });

        while let Some(Ok(json)) = input.next().await {
            self.handle_incoming(&json, output_tx.clone()).await;
        }
    }

    async fn handle_incoming(&self, json: &str, mut output: mpsc::Sender<String>) {
        match serde_json::from_str(json).map_err(|_| Error::parse_error()) {
            Ok(Message::Request(request)) => {
                let server = Arc::clone(&self.server);
                tokio::spawn(async move {
                    let response = handle!(Self::handle_request, server, request, {
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
                handle!(Self::handle_notification, self.server, notification, {
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
            Ok(Message::Response(_)) => unimplemented!(),
            Err(why) => {
                let response = Response::error(why, None);
                let json = serde_json::to_string(&response).unwrap();
                output.send(json).await.unwrap();
            }
        };
    }

    async fn handle_request<'a, H, F, I, O>(request: Request, handler: H) -> Response
    where
        H: Fn(I) -> F + Send + Sync + 'a,
        F: Future<Output = server::Result<O>> + Send,
        I: DeserializeOwned + Send,
        O: Serialize,
    {
        let handle = |json| async move {
            let params: I = serde_json::from_value(json).map_err(|_| Error::deserialize_error())?;
            let result = handler(params).await.map_err(Error::internal_error)?;
            Ok(result)
        };

        match handle(request.params).await {
            Ok(result) => Response::result(json!(result), request.id),
            Err(error) => Response::error(error, Some(request.id)),
        }
    }

    async fn handle_notification<'a, H, F, I>(notification: Notification, handler: H)
    where
        H: Fn(I) -> F + Send + Sync + 'a,
        F: Future<Output = ()> + Send,
        I: DeserializeOwned + Send,
    {
        let error = Error::deserialize_error().message;
        let params = serde_json::from_value(notification.params).expect(&error);
        handler(params).await;
    }
}
