use futures::{
    executor::LocalPool,
    future::{BoxFuture, FutureExt},
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    task::LocalSpawnExt,
};
use indoc::indoc;
use jsonrpc::{Notification, Request};
use language_server::{
    async_trait::async_trait,
    jsonrpc::{Id, Response},
    types::*,
    *,
};
use mockall::mock;
use serde::{de::DeserializeOwned, Serialize};
use sluice::pipe::{pipe, PipeReader};
use std::{fmt::Debug, sync::Arc};

mock! {
    pub LanguageServer {
        fn initialize(
            &self,
            params: InitializeParams,
            client: Arc<dyn LanguageClient>,
        ) -> BoxFuture<'static, Result<InitializeResult>>;

        fn initialized(
            &self,
            params: InitializedParams,
            client: Arc<dyn LanguageClient>
        ) -> BoxFuture<'static, ()>;

        fn shutdown(
            &self,
            params: (),
            client: Arc<dyn LanguageClient>
        ) -> BoxFuture<'static, Result<()>>;
    }
}

#[async_trait]
impl LanguageServer for MockLanguageServer {
    async fn initialize(
        &self,
        params: InitializeParams,
        client: Arc<dyn LanguageClient>,
    ) -> Result<InitializeResult> {
        self.initialize(params, client).await
    }

    async fn initialized(&self, params: InitializedParams, client: Arc<dyn LanguageClient>) {
        self.initialized(params, client).await
    }

    async fn shutdown(&self, params: (), client: Arc<dyn LanguageClient>) -> Result<()> {
        self.shutdown(params, client).await
    }
}

async fn read_message<T>(reader: &mut PipeReader, expected: T)
where
    T: Serialize + DeserializeOwned + Debug + PartialEq,
{
    let length = serde_json::to_string(&expected).unwrap().len();
    let mut length_header = String::new();
    reader.read_line(&mut length_header).await.unwrap();
    assert_eq!(length_header.trim(), format!("Content-Length: {}", length));
    reader.read_line(&mut String::new()).await.unwrap(); // skip newline
    let mut buf = vec![0; length];
    reader.read_exact(&mut buf).await.unwrap();
    println!("{}", String::from_utf8_lossy(&buf).into_owned());
    assert_eq!(serde_json::from_slice::<T>(&buf).unwrap(), expected);
}

#[test]
fn simple_request_success() {
    let mut server = MockLanguageServer::new();
    server
        .expect_initialize()
        .times(1)
        .returning(|_, _| async move { Ok(InitializeResult::default()) }.boxed());

    let mut executor = LocalPool::new();
    let (rx1, mut tx1) = pipe();
    let (mut rx2, tx2) = pipe();

    let service = LanguageService::builder()
        .input(rx1)
        .output(tx2)
        .executor(executor.spawner())
        .server(Arc::new(server))
        .build();

    executor
        .spawner()
        .spawn_local(service.listen())
        .expect("failed to spawn server");

    executor.run_until(async move {
        tx1.write_all(
            indoc!(
                r#"
                    Content-Length: 75

                    {"jsonrpc":"2.0","method":"initialize","id":0,"params":{"capabilities":{}}}
                "#
            )
            .trim()
            .as_bytes(),
        )
        .await
        .unwrap();

        let response = Response::result(
            serde_json::to_value(InitializeResult::default()).unwrap(),
            Id::Number(0),
        );
        read_message(&mut rx2, response).await;
    });
}

#[test]
fn notification_with_client_notification_success() {
    let mut server = MockLanguageServer::new();
    server.expect_initialized().times(1).returning(|_, client| {
        async move {
            let params = LogMessageParams {
                typ: MessageType::Info,
                message: "Hello World!".into(),
            };
            client.log_message(params).await
        }
        .boxed()
    });

    let mut executor = LocalPool::new();
    let (rx1, mut tx1) = pipe();
    let (mut rx2, tx2) = pipe();

    let service = LanguageService::builder()
        .input(rx1)
        .output(tx2)
        .executor(executor.spawner())
        .server(Arc::new(server))
        .build();

    executor
        .spawner()
        .spawn_local(service.listen())
        .expect("failed to spawn server");

    executor.run_until(async move {
        tx1.write_all(
            indoc!(
                r#"
                    Content-Length: 52

                    {"jsonrpc":"2.0","method":"initialized","params":{}}
                "#
            )
            .trim()
            .as_bytes(),
        )
        .await
        .unwrap();

        let notification = Notification::new(
            "window/logMessage".into(),
            serde_json::to_value(LogMessageParams {
                typ: MessageType::Info,
                message: "Hello World!".into(),
            })
            .unwrap(),
        );
        read_message(&mut rx2, notification).await;
    });
}

#[test]
fn request_with_client_request_success() {
    let mut server = MockLanguageServer::new();
    server
        .expect_shutdown()
        .times(1)
        .returning(move |_, client| {
            async move {
                let params = ShowMessageRequestParams {
                    actions: None,
                    message: "Hello World!".into(),
                    typ: MessageType::Info,
                };
                client.show_message_request(params).await.unwrap();
                Ok(())
            }
            .boxed()
        });

    let mut executor = LocalPool::new();
    let (rx1, mut tx1) = pipe();
    let (mut rx2, tx2) = pipe();

    let service = LanguageService::builder()
        .input(rx1)
        .output(tx2)
        .executor(executor.spawner())
        .server(Arc::new(server))
        .build();

    executor
        .spawner()
        .spawn_local(service.listen())
        .expect("failed to spawn server");

    executor.run_until(async move {
        tx1.write_all(
            indoc!(
                r#"
                    Content-Length: 58

                    {"jsonrpc":"2.0","method":"shutdown","id":0,"params":null}
                "#
            )
            .trim()
            .as_bytes(),
        )
        .await
        .unwrap();

        let request = Request::new(
            "window/showMessageRequest".into(),
            serde_json::to_value(ShowMessageRequestParams {
                actions: None,
                message: "Hello World!".into(),
                typ: MessageType::Info,
            })
            .unwrap(),
            Id::Number(0),
        );
        read_message(&mut rx2, request).await;

        tx1.write_all(
            indoc!(
                r#"
                    Content-Length: 38

                    {"jsonrpc":"2.0","id":0,"result":null}
                "#
            )
            .trim()
            .as_bytes(),
        )
        .await
        .unwrap();

        let request = Response::result(serde_json::Value::Null, Id::Number(0));
        read_message(&mut rx2, request).await;
    });
}
