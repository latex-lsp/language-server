use async_executors::TokioTp;
use language_server::{async_trait::async_trait, types::*, *};
use std::{convert::TryFrom, sync::Arc};
use tokio_util::compat::*;

struct Server;

#[async_trait]
impl LanguageServer for Server {
    async fn initialize(
        &self,
        _params: InitializeParams,
        _client: Arc<dyn LanguageClient>,
    ) -> Result<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _params: InitializedParams, client: Arc<dyn LanguageClient>) {
        let params = ShowMessageParams {
            typ: MessageType::Info,
            message: "Hello World!".to_owned(),
        };

        client.show_message(params).await;
    }
}

fn main() {
    let executor = TokioTp::try_from(&mut tokio::runtime::Builder::new())
        .expect("failed to create thread pool");

    executor.block_on(
        LanguageService::builder()
            .server(Arc::new(Server))
            .input(tokio::io::stdin().compat())
            .output(tokio::io::stdout().compat_write())
            .executor(executor.clone())
            .build()
            .listen(),
    );
}
