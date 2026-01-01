//! PipeWire error types.

use thiserror::Error;

/// PipeWire error type.
#[derive(Debug, Error)]
pub enum PwError {
    #[error("PipeWire connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Node creation failed: {0}")]
    NodeCreationFailed(String),

    #[error("Link creation failed: {0}")]
    LinkCreationFailed(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("MainLoop error: {0}")]
    MainLoopError(String),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Volume control failed: {0}")]
    VolumeControlFailed(String),
}

/// Result type for PipeWire operations.
pub type PwResult<T> = Result<T, PwError>;
