mod client;
mod codec;
pub mod jsonrpc;
mod server;

pub use client::LanguageClient;
pub use server::LanguageServer;

pub mod testing {
    //! Helpers to test language servers.
    pub use crate::server::TestLanguageClient;
}

pub use async_trait;
pub use lsp_types as types;

use client::ResponseHandler;
use codec::LspCodec;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use jsonrpc::*;
use server::RequestHandler;
use std::sync::Arc;
use tokio::prelude::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};

/// Represents a service that processes messages according to the
/// [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
pub struct LanguageService<I, O, S> {
    input: I,
    output: O,
    output_tx: mpsc::Sender<String>,
    output_rx: mpsc::Receiver<String>,
    server: Arc<S>,
    client: LanguageClient,
}

impl<I, O, S> LanguageService<I, O, S>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Send + Unpin + 'static,
    S: LanguageServer + Send + Sync + 'static,
{
    /// Creates a new `LspService`.
    pub fn new(input: I, output: O, server: Arc<S>) -> Self {
        let (output_tx, output_rx) = mpsc::channel(0);
        let client = LanguageClient::new(output_tx.clone());

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
            let client = self.client.clone();
            let mut output = self.output_tx.clone();

            match serde_json::from_str(&json).map_err(|_| Error::parse_error()) {
                Ok(Message::Request(request)) => {
                    let server = Arc::clone(&self.server);
                    let mut output = self.output_tx.clone();

                    tokio::spawn(async move {
                        let response = server.handle_request(request, client).await;
                        let json = serde_json::to_string(&response).unwrap();
                        output.send(json).await.unwrap();
                    });
                }
                Ok(Message::Notification(notification)) => {
                    self.server.handle_notification(notification, client).await
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
    }
}
