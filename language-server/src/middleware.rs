use crate::{jsonrpc::*, LanguageClient};
use async_trait::async_trait;
use std::sync::Arc;

/// Allows to do additional work before and/or after processing the message.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Method invoked before an incoming message is being processed.
    async fn on_incoming_message(&self, message: &mut Message, client: Arc<dyn LanguageClient>);

    /// Method invoked before an outgoing response is being sent.
    async fn on_outgoing_response(
        &self,
        request: &Request,
        response: &mut Response,
        client: Arc<dyn LanguageClient>,
    );

    /// Method invoked before an outgoing request is being sent.
    async fn on_outgoing_request(&self, request: &mut Request, client: Arc<dyn LanguageClient>);

    /// Method invoked before an outgoing notification is being sent.
    async fn on_outgoing_notification(
        &self,
        notification: &mut Notification,
        client: Arc<dyn LanguageClient>,
    );
}

#[derive(Clone)]
pub struct AggregateMiddleware {
    pub middlewares: Vec<Arc<dyn Middleware>>,
}

#[async_trait]
impl Middleware for AggregateMiddleware {
    async fn on_incoming_message(&self, message: &mut Message, client: Arc<dyn LanguageClient>) {
        for middleware in &self.middlewares {
            middleware
                .on_incoming_message(message, Arc::clone(&client))
                .await;
        }
    }

    async fn on_outgoing_response(
        &self,
        request: &Request,
        response: &mut Response,
        client: Arc<dyn LanguageClient>,
    ) {
        for middleware in &self.middlewares {
            middleware
                .on_outgoing_response(request, response, Arc::clone(&client))
                .await;
        }
    }

    async fn on_outgoing_request(&self, request: &mut Request, client: Arc<dyn LanguageClient>) {
        for middleware in &self.middlewares {
            middleware
                .on_outgoing_request(request, Arc::clone(&client))
                .await;
        }
    }

    async fn on_outgoing_notification(
        &self,
        notification: &mut Notification,
        client: Arc<dyn LanguageClient>,
    ) {
        for middleware in &self.middlewares {
            middleware
                .on_outgoing_notification(notification, Arc::clone(&client))
                .await;
        }
    }
}

/// Middleware that logs every incoming and outgoing message.
///
/// Each message is logged with the "trace" level.
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    fn log_message<T>(message: T, text: &str)
    where
        T: serde::Serialize,
    {
        let json = serde_json::to_string_pretty(&message).expect("failed to serialize value");
        log::trace!("{}:\n{}\n", text, json);
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn on_incoming_message(&self, message: &mut Message, _client: Arc<dyn LanguageClient>) {
        let kind = match message {
            Message::Request(_) => "request",
            Message::Notification(_) => "notification",
            Message::Response(_) => "response",
        };

        Self::log_message(message, &format!("Received {} (->)", kind));
    }

    async fn on_outgoing_response(
        &self,
        _request: &Request,
        response: &mut Response,
        _client: Arc<dyn LanguageClient>,
    ) {
        Self::log_message(response, "Sent response (<-)");
    }

    async fn on_outgoing_request(&self, request: &mut Request, _client: Arc<dyn LanguageClient>) {
        Self::log_message(request, "Sent request (<-)");
    }

    async fn on_outgoing_notification(
        &self,
        notification: &mut Notification,
        _client: Arc<dyn LanguageClient>,
    ) {
        Self::log_message(notification, "Sent notification (<-)");
    }
}
