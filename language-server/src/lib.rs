//! A library to implement language servers in Rust.
//!
//! # Example
//!
//! A simple language server using the [Tokio](https://tokio.rs/) runtime:
//!
//! ```no_run
//! use async_executors::TokioTp;
//! use language_server::{async_trait::async_trait, jsonrpc::Result, types::*, *};
//! use std::convert::TryFrom;
//! use tokio_util::compat::*;
//!
//! struct Server;
//!
//! #[async_trait]
//! impl LanguageServer for Server {
//!     async fn initialize(
//!         &self,
//!         _params: InitializedParams,
//!         _client: &dyn LanguageClient,
//!     ) -> Result<InitializeResult> {
//!         Ok(InitializeResult::default())
//!     }
//!
//!     async fn initialized(&self, _params: InitializedParams, client: &dyn LanguageClient) {
//!         let params = ShowMessageParams {
//!             typ: MessageType::Info,
//!             message: "Hello World!".to_owned(),
//!         };
//!
//!         client.show_message(params).await;
//!     }
//! }
//!
//! fn main() {
//!     let stdin = tokio::io::stdin().compat();
//!     let stdout = tokio::io::stdout().compat_write();
//!     let executor = TokioTp::try_from(&mut tokio::runtime::Builder::new())
//!         .expect("failed to create thread pool");
//!
//!     let service = LanguageService::new(stdin, stdout, Server, executor.clone());
//!     executor.block_on(service.listen());
//! }
//! ```
mod client;
mod codec;
pub mod jsonrpc;
mod server;

pub use client::LanguageClient;
pub use server::{LanguageServer, Middleware};

pub use async_trait;
pub use lsp_types as types;

use crate::{
    client::{LanguageClientImpl, ResponseHandler},
    codec::LspCodec,
    jsonrpc::*,
    server::{NoOpMiddleware, RequestHandler},
};
use futures::{
    channel::mpsc,
    sink::SinkExt,
    stream::StreamExt,
    task::{Spawn, SpawnExt},
    AsyncRead, AsyncWrite,
};
use futures_codec::{FramedRead, FramedWrite};
use std::sync::Arc;

/// Represents a service that processes messages according to the
/// [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
pub struct LanguageService<I, O, S, E> {
    input: I,
    output: O,
    output_tx: mpsc::Sender<String>,
    output_rx: mpsc::Receiver<String>,
    server: Arc<S>,
    client: LanguageClientImpl,
    middleware: Arc<dyn Middleware>,
    executor: E,
}

impl<I, O, S, E> LanguageService<I, O, S, E>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Send + Unpin + 'static,
    S: LanguageServer + Send + Sync + 'static,
    E: Spawn + Clone,
{
    /// Creates a new `LspService`.
    pub fn new(input: I, output: O, server: S, executor: E) -> Self {
        let middleware = Arc::new(NoOpMiddleware);
        Self::with_middleware(input, output, Arc::new(server), executor, middleware)
    }

    /// Creates a new `LspService` with the given `Middleware`.
    pub fn with_middleware(
        input: I,
        output: O,
        server: Arc<S>,
        executor: E,
        middleware: Arc<dyn Middleware>,
    ) -> Self {
        let (output_tx, output_rx) = mpsc::channel(0);
        let client = LanguageClientImpl::new(output_tx.clone());

        Self {
            input,
            output,
            output_tx,
            output_rx,
            server,
            client,
            executor,
            middleware,
        }
    }

    /// Starts the service and processes messages.
    /// It is guaranteed that all notifications are processed in order.
    pub async fn listen(self) {
        let output = self.output;
        let mut output_rx = self.output_rx;
        self.executor
            .spawn(async move {
                let mut output = FramedWrite::new(output, LspCodec);
                loop {
                    let message = output_rx.next().await.unwrap();
                    output.send(message).await.unwrap();
                }
            })
            .expect("failed to spawn future");

        let mut input = FramedRead::new(self.input, LspCodec);
        while let Some(Ok(json)) = input.next().await {
            let server = Arc::clone(&self.server);
            let client = self.client.clone();
            let mut output = self.output_tx.clone();
            let executor = self.executor.clone();
            let middleware = Arc::clone(&self.middleware);

            match serde_json::from_str(&json) {
                Ok(message) => {
                    Self::handle_message(server, client, output, executor, middleware, message)
                        .await
                }
                Err(_) => {
                    let response = Response::error(Error::parse_error(), None);
                    let json = serde_json::to_string(&response).unwrap();
                    output.send(json).await.unwrap();
                }
            };
        }
    }

    async fn handle_message(
        server: Arc<S>,
        client: LanguageClientImpl,
        mut output: mpsc::Sender<String>,
        executor: E,
        middleware: Arc<dyn Middleware>,
        message: Message,
    ) {
        middleware.before_message(&message).await;

        match message.clone() {
            Message::Request(request) => {
                executor
                    .spawn(async move {
                        let response = server.handle_request(request, &client).await;
                        let json = serde_json::to_string(&response).unwrap();
                        output.send(json).await.unwrap();
                        middleware.after_message(&message, Some(&response)).await;
                    })
                    .expect("failed to spawn future");
            }
            Message::Notification(notification) => {
                server.handle_notification(notification, &client).await;
                middleware.after_message(&message, None).await;
            }
            Message::Response(response) => {
                client.handle(response).await;
                middleware.after_message(&message, None).await;
            }
        };
    }
}
