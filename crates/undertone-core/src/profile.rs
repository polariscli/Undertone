//! Profile management for saving/loading mixer configurations.

use serde::{Deserialize, Serialize};

use crate::channel::ChannelState;
use crate::mixer::MixerState;
use crate::routing::RouteRule;

/// A saved mixer profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether this is the default profile
    pub is_default: bool,
    /// Channel states snapshot
    pub channels: Vec<ProfileChannel>,
    /// Routing rules snapshot
    pub routes: Vec<RouteRule>,
    /// Mixer state snapshot
    pub mixer: MixerState,
}

/// Channel state within a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileChannel {
    /// Channel name
    pub name: String,
    /// Stream mix volume
    pub stream_volume: f32,
    /// Stream mix muted
    pub stream_muted: bool,
    /// Monitor mix volume
    pub monitor_volume: f32,
    /// Monitor mix muted
    pub monitor_muted: bool,
}

impl From<&ChannelState> for ProfileChannel {
    fn from(state: &ChannelState) -> Self {
        Self {
            name: state.config.name.clone(),
            stream_volume: state.stream_volume,
            stream_muted: state.stream_muted,
            monitor_volume: state.monitor_volume,
            monitor_muted: state.monitor_muted,
        }
    }
}

impl Profile {
    /// Create a new empty profile.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            is_default: false,
            channels: Vec::new(),
            routes: Vec::new(),
            mixer: MixerState::default(),
        }
    }

    /// Create the default profile.
    #[must_use]
    pub fn default_profile() -> Self {
        Self {
            name: "Default".to_string(),
            description: Some("Default mixer configuration".to_string()),
            is_default: true,
            channels: Vec::new(),
            routes: crate::routing::default_routes(),
            mixer: MixerState::default(),
        }
    }
}
