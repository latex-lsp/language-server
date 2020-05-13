use async_executors::TokioTp;
use language_server::{async_trait::async_trait, jsonrpc::Result, types::*, *};
use std::convert::TryFrom;
use tokio_util::compat::*;

struct Server;

#[async_trait]
impl LanguageServer for Server {
    async fn initialize(
        &self,
        _params: InitializedParams,
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

    let stdin = tokio::io::stdin().compat();
    let stdout = tokio::io::stdout().compat_write();
    let executor = TokioTp::try_from(&mut tokio::runtime::Builder::new())
        .expect("failed to create thread pool");

    let service = LanguageService::new(stdin, stdout, Server, executor.clone());
    executor.block_on(service.listen());
}