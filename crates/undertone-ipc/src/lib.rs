//! Undertone IPC - Unix socket protocol and client library.
//!
//! This crate defines the communication protocol between the daemon and UI,
//! as well as providing a client library for connecting to the daemon.

pub mod client;
pub mod error;
pub mod events;
pub mod messages;
pub mod server;

pub use client::IpcClient;
pub use error::{IpcError, IpcResult};
pub use events::{
    AppDiscoveredData, ChannelMuteChangedData, ChannelVolumeChangedData, DeviceConnectedData,
    ErrorData, Event, EventType, LevelsData,
};
pub use messages::{Method, Request, Response};
pub use server::IpcServer;

use std::path::PathBuf;

/// Get the default socket path.
///
/// Uses `$XDG_RUNTIME_DIR/undertone/daemon.sock` or falls back to
/// `/run/user/$UID/undertone/daemon.sock`.
#[must_use]
#[allow(unsafe_code)] // libc::getuid() is safe to call
pub fn socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("undertone/daemon.sock")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/run/user/{uid}/undertone/daemon.sock"))
    }
}
