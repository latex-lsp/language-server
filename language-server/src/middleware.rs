//! Middleware support for the `LanguageService`.
use crate::jsonrpc::*;
use async_trait::async_trait;

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

/// Empty middleware implementation.
pub struct NoOpMiddleware;

#[async_trait]
impl Middleware for NoOpMiddleware {
    async fn on_incoming_message(&self, _message: &mut Message) {}

    async fn on_outgoing_response(&self, _request: &Request, _response: &mut Response) {}

    async fn on_outgoing_request(&self, _request: &mut Request) {}

    async fn on_outgoing_notification(&self, _notification: &mut Notification) {}
}
