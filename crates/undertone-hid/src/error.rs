//! HID error types.

use thiserror::Error;

/// HID error type.
#[derive(Debug, Error)]
pub enum HidError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Permission denied - check udev rules")]
    PermissionDenied,

    #[error("USB error: {0}")]
    UsbError(String),

    #[error("ALSA error: {0}")]
    AlsaError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for HID operations.
pub type HidResult<T> = Result<T, HidError>;
