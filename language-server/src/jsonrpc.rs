//! Types for JSON-RPC messages.
use serde::{Deserialize, Serialize};
use serde_repr::*;

const PROTOCOL_VERSION: &str = "2.0";

/// The identifier that is used in an JSON-RPC message.
#[derive(Debug, Eq, Hash, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    Number(u64),
    String(String),
}

/// The JSON-RPC error codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,
    RequestCancelled = -32800,
}

/// The error type for JSON-RPC messages.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    /// Returns an `Error` with the [`ParseError`](enum.ErrorCode.html#variant.ParseError) error code.
    pub fn parse_error() -> Self {
        Self {
            code: ErrorCode::ParseError,
            message: "Could not parse the input".to_owned(),
            data: None,
        }
    }

    /// Returns an `Error` with the [`MethodNotFound`](enum.ErrorCode.html#variant.MethodNotFound) error code.
    pub fn method_not_found_error() -> Self {
        Self {
            code: ErrorCode::MethodNotFound,
            message: "Method not found".to_owned(),
            data: None,
        }
    }

    /// Returns an `Error` with the [`InvalidParams`](enum.ErrorCode.html#variant.InvalidParams) error code.
    pub fn deserialize_error() -> Self {
        Self {
            code: ErrorCode::InvalidParams,
            message: "Could not deserialize parameter object".to_owned(),
            data: None,
        }
    }

    /// Returns an `Error` with the [`internal_error`](enum.ErrorCode.html#variant.internal_error) error code.
    pub fn internal_error(message: String) -> Self {
        Self {
            code: ErrorCode::InternalError,
            message,
            data: None,
        }
    }
}

/// A specialized Result type for JSON-RPC operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The request type for JSON-RPC messages.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Id,
}

impl Request {
    // Creates a new `Request`.
    pub fn new(method: String, params: serde_json::Value, id: Id) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            method,
            params,
            id,
        }
    }
}

/// The response type for JSON-RPC messages.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Response {
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,

    pub id: Option<Id>,
}

impl Response {
    /// Returns an `Response` with the given `result`.
    pub fn result(result: serde_json::Value, id: Id) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    /// Returns an `Response` with the given `error`.
    pub fn error(error: Error, id: Option<Id>) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// The notification type for JSON-RPC messages.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl Notification {
    // Creates a new `Notification`.
    pub fn new(method: String, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: PROTOCOL_VERSION.to_owned(),
            method,
            params,
        }
    }
}

/// Represents a JSON-RPC message.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}
