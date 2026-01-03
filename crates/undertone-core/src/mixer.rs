//! Mixer state and mix types.

use serde::{Deserialize, Serialize};

/// Type of audio mix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MixType {
    /// Stream mix - sent to streaming/recording applications
    #[default]
    Stream,
    /// Monitor mix - sent to local headphone output
    Monitor,
}

impl MixType {
    /// Get the node name prefix for this mix type.
    #[must_use]
    pub fn node_prefix(&self) -> &'static str {
        match self {
            Self::Stream => "ut-stream",
            Self::Monitor => "ut-monitor",
        }
    }
}

/// State of the overall mixer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct MixerState {
    /// Master volume for stream mix
    pub stream_master_volume: f32,
    /// Master mute for stream mix
    pub stream_master_muted: bool,
    /// Master volume for monitor mix
    pub monitor_master_volume: f32,
    /// Master mute for monitor mix
    pub monitor_master_muted: bool,
    /// Whether microphone is included in stream mix
    pub mic_to_stream: bool,
    /// Microphone level in stream mix (0.0 - 1.0)
    pub mic_stream_volume: f32,
    /// Whether microphone is included in monitor mix (sidetone)
    pub mic_to_monitor: bool,
    /// Microphone level in monitor mix
    pub mic_monitor_volume: f32,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            stream_master_volume: 1.0,
            stream_master_muted: false,
            monitor_master_volume: 1.0,
            monitor_master_muted: false,
            mic_to_stream: true,
            mic_stream_volume: 1.0,
            mic_to_monitor: false,
            mic_monitor_volume: 0.0,
        }
    }
}
