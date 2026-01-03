//! Audio channel definitions and state.

use serde::{Deserialize, Serialize};

/// Configuration for a virtual audio channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Internal name (e.g., "system", "voice", "music")
    pub name: String,
    /// Display name shown in UI
    pub display_name: String,
    /// Icon identifier
    pub icon: Option<String>,
    /// Color hex code
    pub color: Option<String>,
    /// Sort order in the mixer
    pub sort_order: i32,
    /// Whether this is a system-defined channel
    pub is_system: bool,
}

impl ChannelConfig {
    /// Create a new system channel configuration.
    #[must_use]
    pub fn system(name: &str, display_name: &str, sort_order: i32) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            icon: None,
            color: None,
            sort_order,
            is_system: true,
        }
    }

    /// Get the PipeWire node name for this channel.
    #[must_use]
    pub fn node_name(&self) -> String {
        format!("ut-ch-{}", self.name)
    }

    /// Get the PipeWire node name for this channel's stream volume filter.
    #[must_use]
    pub fn stream_vol_node_name(&self) -> String {
        format!("ut-ch-{}-stream-vol", self.name)
    }

    /// Get the PipeWire node name for this channel's monitor volume filter.
    #[must_use]
    pub fn monitor_vol_node_name(&self) -> String {
        format!("ut-ch-{}-monitor-vol", self.name)
    }
}

/// Default system channels.
pub fn default_channels() -> Vec<ChannelConfig> {
    vec![
        ChannelConfig::system("system", "System", 0),
        ChannelConfig::system("voice", "Voice", 1),
        ChannelConfig::system("music", "Music", 2),
        ChannelConfig::system("browser", "Browser", 3),
        ChannelConfig::system("game", "Game", 4),
    ]
}

/// Runtime state for a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    /// Channel configuration
    pub config: ChannelConfig,
    /// Volume level for stream mix (0.0 - 1.0)
    pub stream_volume: f32,
    /// Muted in stream mix
    pub stream_muted: bool,
    /// Volume level for monitor mix (0.0 - 1.0)
    pub monitor_volume: f32,
    /// Muted in monitor mix
    pub monitor_muted: bool,
    /// Current audio level (left channel, 0.0 - 1.0)
    pub level_left: f32,
    /// Current audio level (right channel, 0.0 - 1.0)
    pub level_right: f32,
    /// PipeWire node ID (if created)
    pub node_id: Option<u32>,
}

impl ChannelState {
    /// Create a new channel state from configuration.
    #[must_use]
    pub fn new(config: ChannelConfig) -> Self {
        Self {
            config,
            stream_volume: 1.0,
            stream_muted: false,
            monitor_volume: 1.0,
            monitor_muted: false,
            level_left: 0.0,
            level_right: 0.0,
            node_id: None,
        }
    }
}

/// A channel with its full state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Unique identifier
    pub id: i64,
    /// Channel state
    pub state: ChannelState,
}
