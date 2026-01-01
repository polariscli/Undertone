//! IPC error types.

use thiserror::Error;

/// IPC error type.
#[derive(Debug, Error)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Request timeout")]
    Timeout,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Invalid message format")]
    InvalidMessage,

    #[error("Channel closed")]
    ChannelClosed,
}

/// Result type for IPC operations.
pub type IpcResult<T> = Result<T, IpcError>;
