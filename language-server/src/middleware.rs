use crate::jsonrpc::*;
use async_trait::async_trait;
use std::sync::Arc;

/// Allows to do additional work before and/or after processing the message.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Method invoked before an incoming message is being processed.
    async fn on_incoming_message(&self, message: &mut Message);

    /// Method invoked before an outgoing response is being sent.
    async fn on_outgoing_response(&self, request: &Request, response: &mut Response);

    /// Method invoked before an outgoing request is being sent.
    async fn on_outgoing_request(&self, request: &mut Request);

    /// Method invoked before an outgoing notification is being sent.
    async fn on_outgoing_notification(&self, notification: &mut Notification);
}

#[derive(Clone)]
pub struct AggregateMiddleware {
    pub middlewares: Vec<Arc<dyn Middleware>>,
}

#[async_trait]
impl Middleware for AggregateMiddleware {
    async fn on_incoming_message(&self, message: &mut Message) {
        for middleware in &self.middlewares {
            middleware.on_incoming_message(message).await;
        }
    }

    async fn on_outgoing_response(&self, request: &Request, response: &mut Response) {
        for middleware in &self.middlewares {
            middleware.on_outgoing_response(request, response).await;
        }
    }

    async fn on_outgoing_request(&self, request: &mut Request) {
        for middleware in &self.middlewares {
            middleware.on_outgoing_request(request).await;
        }
    }

    async fn on_outgoing_notification(&self, notification: &mut Notification) {
        for middleware in &self.middlewares {
            middleware.on_outgoing_notification(notification).await;
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
    async fn on_incoming_message(&self, message: &mut Message) {
        let kind = match message {
            Message::Request(_) => "request",
            Message::Notification(_) => "notification",
            Message::Response(_) => "response",
        };

        Self::log_message(message, &format!("Received {} (->)", kind));
    }

    async fn on_outgoing_response(&self, _request: &Request, response: &mut Response) {
        Self::log_message(response, "Sent response (<-)");
    }

    async fn on_outgoing_request(&self, request: &mut Request) {
        Self::log_message(request, "Sent request (<-)");
    }

    async fn on_outgoing_notification(&self, notification: &mut Notification) {
        Self::log_message(notification, "Sent notification (<-)");
    }
}
