use async_executors::AsyncStd;
use language_server::{async_trait::async_trait, types::*, *};
use std::sync::Arc;

struct Server;

#[async_trait]
impl LanguageServer for Server {
    async fn initialize(
        &self,
        _params: InitializeParams,
        _client: &dyn LanguageClient,
    ) -> Result<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _params: InitializedParams, client: &dyn LanguageClient) {
        let params = ShowMessageParams {
            typ: MessageType::Info,
            message: "Hello World!".to_owned(),
        };

        client.show_message(params).await;
    }
}

fn main() {
    stderrlog::new()
        .module(module_path!())
        .module("language_server")
        .verbosity(5)
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .expect("failed to init logger");

    let stdin = async_std::io::stdin();
    let stdout = async_std::io::stdout();
    let service = LanguageService::new(stdin, stdout, Arc::new(Server), AsyncStd);
    AsyncStd::block_on(service.listen());
}
