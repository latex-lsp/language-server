#![cfg_attr(docsrs, feature(doc_cfg))]

//! A library to implement asynchronous language servers in Rust.
//!
//! It features full support of the
//! [Language Server Protocol 3.15](https://microsoft.github.io/language-server-protocol/specifications/specification-3-15/)
//! and is designed to be independent of the underlying transport layer and the used async executor.
//!
//! # Example
//!
//! A simple language server using the [Tokio](https://tokio.rs/) runtime:
//!
//! ```no_run
//! use async_executors::TokioTp;
//! use language_server::{async_trait::async_trait, types::*, *};
//! use std::{convert::TryFrom, sync::Arc};
//! use tokio_util::compat::*;
//!
//! struct Server;
//!
//! #[async_trait]
//! impl LanguageServer for Server {
//!     async fn initialize(
//!         &self,
//!         _params: InitializeParams,
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
//!     let service = LanguageService::new(stdin, stdout, Arc::new(Server), executor.clone());
//!     executor.block_on(service.listen());
//! }
//! ```
mod client;
mod codec;
pub mod jsonrpc;
mod middleware;
mod server;

pub use client::LanguageClient;
pub use jsonrpc::Result;
pub use middleware::Middleware;
pub use server::LanguageServer;

pub use async_trait;
pub use lsp_types as types;

use crate::{
    client::{LanguageClientImpl, ResponseHandler},
    codec::LspCodec,
    jsonrpc::*,
    middleware::AggregateMiddleware,
    server::RequestHandler,
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
    output_tx: mpsc::Sender<Message>,
    output_rx: mpsc::Receiver<Message>,
    server: Arc<S>,
    client: LanguageClientImpl,
    middleware: AggregateMiddleware,
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
    pub fn new(input: I, output: O, server: Arc<S>, executor: E) -> Self {
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
            middleware: AggregateMiddleware::new(),
        }
    }

    /// Attaches a middleware to the service.
    pub fn middleware(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middleware.middlewares.push(middleware);
        self
    }

    /// Starts the service and processes messages.
    /// It is guaranteed that all notifications are processed in order.
    pub async fn listen(self) {
        let output = self.output;
        let mut output_rx = self.output_rx;
        let middleware = self.middleware.clone();
        self.executor
            .spawn(async move {
                let mut output = FramedWrite::new(output, LspCodec);
                while let Some(mut message) = output_rx.next().await {
                    match &mut message {
                        Message::Request(ref mut request) => {
                            middleware.on_outgoing_request(request).await;
                        }
                        Message::Notification(ref mut notification) => {
                            middleware.on_outgoing_notification(notification).await;
                        }
                        Message::Response(_) => {}
                    };

                    let json =
                        serde_json::to_string(&message).expect("failed to serialize message");
                    output.send(json).await.expect("failed to send message");
                }
            })
            .expect("failed to spawn future");

        let mut input = FramedRead::new(self.input, LspCodec);
        while let Some(Ok(json)) = input.next().await {
            let server = Arc::clone(&self.server);
            let client = self.client.clone();
            let mut output = self.output_tx.clone();
            let executor = self.executor.clone();
            let middleware = self.middleware.clone();

            match serde_json::from_str(&json) {
                Ok(message) => {
                    Self::handle_message(server, client, output, executor, middleware, message)
                        .await
                }
                Err(_) => {
                    let response = Response::error(Error::parse_error(), None);
                    output.send(Message::Response(response)).await.unwrap();
                }
            };
        }
    }

    async fn handle_message(
        server: Arc<S>,
        client: LanguageClientImpl,
        mut output: mpsc::Sender<Message>,
        executor: E,
        middleware: AggregateMiddleware,
        mut message: Message,
    ) {
        middleware.on_incoming_message(&mut message).await;

        match message {
            Message::Request(request) => {
                executor
                    .spawn(async move {
                        let mut response = server.handle_request(request.clone(), &client).await;
                        middleware
                            .on_outgoing_response(&request, &mut response)
                            .await;

                        output.send(Message::Response(response)).await.unwrap();
                    })
                    .expect("failed to spawn future");
            }
            Message::Notification(notification) => {
                server.handle_notification(notification, &client).await;
            }
            Message::Response(response) => {
                client.handle(response).await;
            }
        };
    }
}
