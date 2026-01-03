//! IPC event types (server to client).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use undertone_core::mixer::MixType;

/// Event sent from daemon to subscribed clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event type
    pub event: EventType,
    /// Event data
    pub data: Value,
}

/// Types of events that can be subscribed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Daemon state changed
    StateChanged,
    /// Channel volume changed
    ChannelVolumeChanged,
    /// Channel mute state changed
    ChannelMuteChanged,
    /// Audio levels updated (for VU meters)
    LevelsUpdated,
    /// App routing changed
    AppRouteChanged,
    /// New audio app discovered
    AppDiscovered,
    /// Audio app removed
    AppRemoved,
    /// Wave:3 device connected
    DeviceConnected,
    /// Wave:3 device disconnected
    DeviceDisconnected,
    /// Microphone mute state changed
    MicMuteChanged,
    /// Profile changed
    ProfileChanged,
    /// Error occurred
    Error,
}

/// Channel volume changed event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelVolumeChangedData {
    pub channel: String,
    pub mix: MixType,
    pub volume: f32,
}

/// Channel mute changed event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMuteChangedData {
    pub channel: String,
    pub mix: MixType,
    pub muted: bool,
}

/// Audio levels update data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelsData {
    /// Per-channel levels: (`channel_name`, left, right)
    pub channels: Vec<(String, f32, f32)>,
    /// Master output levels
    pub master: (f32, f32),
}

/// App discovered event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDiscoveredData {
    pub app_id: u32,
    pub name: String,
    pub binary: Option<String>,
    pub pid: Option<u32>,
    pub channel: String,
}

/// Device connected event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConnectedData {
    pub serial: String,
}

/// Error event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub code: i32,
    pub message: String,
    pub source: String,
}
