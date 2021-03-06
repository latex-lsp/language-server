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
//!         _client: Arc<dyn LanguageClient>,
//!     ) -> Result<InitializeResult> {
//!         Ok(InitializeResult::default())
//!     }
//!
//!     async fn initialized(&self, _params: InitializedParams, client: Arc<dyn LanguageClient>) {
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
//!     let executor = TokioTp::try_from(&mut tokio::runtime::Builder::new())
//!         .expect("failed to create thread pool");
//!
//!     executor.block_on(
//!         LanguageService::builder()
//!             .server(Arc::new(Server))
//!             .input(tokio::io::stdin().compat())
//!             .output(tokio::io::stdout().compat_write())
//!             .executor(executor.clone())
//!             .build()
//!             .listen(),
//!     );
//! }
//! ```
mod client;
mod codec;
pub mod jsonrpc;
mod middleware;
mod server;

pub use client::LanguageClient;
pub use jsonrpc::Result;
pub use middleware::{LoggingMiddleware, Middleware};
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
use typed_builder::TypedBuilder;

/// Represents a service that processes messages according to the
/// [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).
#[builder(builder_type_doc = "A builder to construct a `LanguageService`.")]
#[builder(builder_method_doc = "Returns a builder for constructing a new `LanguageService`.")]
#[derive(TypedBuilder)]
pub struct LanguageService<I, O, S, E> {
    #[builder(setter(doc = "Sets the input stream for the service."))]
    input: I,

    #[builder(setter(doc = "Sets the output sink for the service."))]
    output: O,

    #[builder(setter(doc = "Sets the language server for the service."))]
    server: Arc<S>,

    #[builder(setter(doc = "Sets the executor on which futures are spawned."))]
    executor: E,

    #[builder(default)]
    #[builder(setter(doc = "Attaches multiple middlewares to the service."))]
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl<I, O, S, E> LanguageService<I, O, S, E>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Send + Unpin + 'static,
    S: LanguageServer + Send + Sync + 'static,
    E: Spawn + Clone,
{
    /// Starts the service and processes messages.
    /// It is guaranteed that all notifications are processed in order.
    pub async fn listen(self) {
        let (output_tx, mut output_rx) = mpsc::channel(0);
        let client = Arc::new(LanguageClientImpl::new(output_tx.clone()));
        let output = self.output;
        let middleware = AggregateMiddleware {
            middlewares: self.middlewares,
        };
        {
            let middleware = middleware.clone();
            let client = Arc::clone(&client);
            self.executor
                .spawn(async move {
                    let mut output = FramedWrite::new(output, LspCodec);
                    while let Some(mut message) = output_rx.next().await {
                        match &mut message {
                            Message::Request(ref mut request) => {
                                middleware
                                    .on_outgoing_request(request, client.clone())
                                    .await;
                            }
                            Message::Notification(ref mut notification) => {
                                middleware
                                    .on_outgoing_notification(notification, client.clone())
                                    .await;
                            }
                            Message::Response(_) => {}
                        };

                        let json =
                            serde_json::to_string(&message).expect("failed to serialize message");
                        output.send(json).await.expect("failed to send message");
                    }
                })
                .expect("failed to spawn future");
        }

        let mut input = FramedRead::new(self.input, LspCodec);
        while let Some(Ok(json)) = input.next().await {
            let server = Arc::clone(&self.server);
            let client = Arc::clone(&client);
            let mut output = output_tx.clone();
            let executor = self.executor.clone();
            let middleware = middleware.clone();

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
        client: Arc<LanguageClientImpl>,
        mut output: mpsc::Sender<Message>,
        executor: E,
        middleware: AggregateMiddleware,
        mut message: Message,
    ) {
        middleware
            .on_incoming_message(&mut message, client.clone())
            .await;

        match message {
            Message::Request(request) => {
                let client = client.clone();
                executor
                    .spawn(async move {
                        let mut response =
                            server.handle_request(request.clone(), client.clone()).await;
                        middleware
                            .on_outgoing_response(&request, &mut response, client)
                            .await;

                        output.send(Message::Response(response)).await.unwrap();
                    })
                    .expect("failed to spawn future");
            }
            Message::Notification(notification) => {
                server.handle_notification(notification, client).await;
            }
            Message::Response(response) => {
                client.handle(response).await;
            }
        };
    }
}
