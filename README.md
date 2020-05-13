[![CI](https://github.com/latex-lsp/language-server/workflows/CI/badge.svg)](https://github.com/latex-lsp/language-server/actions)
[![Coverage](https://codecov.io/gh/latex-lsp/language-server/branch/master/graph/badge.svg)](https://codecov.io/gh/latex-lsp/language-server)
[![Rust](https://img.shields.io/badge/rustc-1.39%2B-blue)](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html)
[![LSP](https://img.shields.io/badge/lsp-3.15-blue)](https://microsoft.github.io/language-server-protocol/specifications/specification-3-15/)
[![crates.io](https://img.shields.io/crates/v/language-server)](https://crates.io/crates/language-server)
[![docs.rs](https://docs.rs/language-server/badge.svg)](https://docs.rs/language-server)

# language-server

A library to implement asynchronous language servers in Rust. It features

- Full server and client support of the
  [Language Server Protocol 3.15](https://microsoft.github.io/language-server-protocol/specifications/specification-3-15/).
- Independent of the underlying transport layer and the used async executor.

## Example

A simple language server using the [Tokio](https://tokio.rs/) runtime:

```rust
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
    let stdin = tokio::io::stdin().compat();
    let stdout = tokio::io::stdout().compat_write();
    let executor = TokioTp::try_from(&mut tokio::runtime::Builder::new())
        .expect("failed to create thread pool");

    let service = LanguageService::new(stdin, stdout, Server, executor.clone());
    executor.block_on(service.listen());
}
```

More examples can be found [here](examples).
