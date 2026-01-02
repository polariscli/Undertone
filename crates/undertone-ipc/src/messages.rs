//! IPC message types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use undertone_core::mixer::MixType;

/// Request envelope sent from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request ID for matching responses
    pub id: u64,
    /// The method to invoke
    pub method: Method,
}

/// Response envelope sent from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID this is responding to
    pub id: u64,
    /// Result of the request
    pub result: Result<Value, ErrorInfo>,
}

/// Error information in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
}

impl ErrorInfo {
    /// Create a new error.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

/// Methods that can be invoked via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Method {
    // State queries
    /// Get the current daemon state snapshot
    GetState,
    /// Get all channels
    GetChannels,
    /// Get a specific channel by name
    GetChannel { name: String },
    /// Get all active apps
    GetApps,
    /// Get all profiles
    GetProfiles,
    /// Get a specific profile by name
    GetProfile { name: String },
    /// Get device connection status
    GetDeviceStatus,
    /// Get diagnostic information
    GetDiagnostics,

    // Channel control
    /// Set volume for a channel in a specific mix
    SetChannelVolume { channel: String, mix: MixType, volume: f32 },
    /// Set mute state for a channel in a specific mix
    SetChannelMute { channel: String, mix: MixType, muted: bool },

    // Master volume control
    /// Set master volume for a mix (0.0 - 1.0)
    SetMasterVolume { mix: MixType, volume: f32 },
    /// Set master mute state for a mix
    SetMasterMute { mix: MixType, muted: bool },

    // App routing
    /// Route an app to a channel
    SetAppRoute { app_pattern: String, channel: String },
    /// Remove an app route
    RemoveAppRoute { app_pattern: String },

    // Profile management
    /// Save current state as a profile
    SaveProfile { name: String },
    /// Load a saved profile
    LoadProfile { name: String },
    /// Delete a profile
    DeleteProfile { name: String },

    // Device control
    /// Set microphone gain (0.0 - 1.0)
    SetMicGain { gain: f32 },
    /// Set microphone mute state
    SetMicMute { muted: bool },

    // Subscriptions
    /// Subscribe to event types
    Subscribe { events: Vec<String> },
    /// Unsubscribe from event types
    Unsubscribe { events: Vec<String> },

    // System
    /// Request graceful shutdown
    Shutdown,
    /// Force reconciliation of PipeWire state
    Reconcile,
}
