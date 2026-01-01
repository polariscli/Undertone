//! Error types for Undertone core.

use thiserror::Error;

/// Core error type for Undertone operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid channel name: {0}")]
    InvalidChannelName(String),

    #[error("Invalid volume value: {0} (must be 0.0-1.0)")]
    InvalidVolume(f32),

    #[error("Route pattern error: {0}")]
    RoutePatternError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type alias for Undertone core operations.
pub type Result<T> = std::result::Result<T, Error>;
