//! Commands for state mutations.
//!
//! Commands are returned by IPC handlers to request state changes
//! that the main daemon loop can process with mutable access.

use crate::mixer::MixType;

/// A command representing a state mutation request.
#[derive(Debug, Clone)]
pub enum Command {
    /// Set volume for a channel in a specific mix
    SetChannelVolume { channel: String, mix: MixType, volume: f32 },
    /// Set mute state for a channel in a specific mix
    SetChannelMute { channel: String, mix: MixType, muted: bool },
    /// Route an app to a channel
    SetAppRoute { app_pattern: String, channel: String },
    /// Remove an app route
    RemoveAppRoute { app_pattern: String },
    /// Save current state as a profile
    SaveProfile { name: String },
    /// Load a saved profile
    LoadProfile { name: String },
    /// Delete a profile
    DeleteProfile { name: String },
    /// Set microphone gain
    SetMicGain { gain: f32 },
    /// Set microphone mute state
    SetMicMute { muted: bool },
    /// Trigger reconciliation
    Reconcile,
    /// Request shutdown
    Shutdown,
}
