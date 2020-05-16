use async_executors::AsyncStd;
use language_server::{async_trait::async_trait, types::*, *};
use std::sync::Arc;

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
    AsyncStd::block_on(
        LanguageService::builder()
            .server(Arc::new(Server))
            .input(async_std::io::stdin())
            .output(async_std::io::stdout())
            .executor(AsyncStd)
            .build()
            .listen(),
    );
}
