//! UI state management.

use arc_swap::ArcSwap;
use std::sync::Arc;
use undertone_core::channel::ChannelState;
use undertone_core::mixer::MixType;

/// Thread-safe UI state.
#[derive(Clone)]
pub struct UiState {
    inner: Arc<ArcSwap<UiStateInner>>,
}

#[derive(Clone, Default)]
struct UiStateInner {
    /// Whether connected to daemon
    connected: bool,
    /// Device serial number
    device_serial: Option<String>,
    /// Active profile name
    active_profile: String,
    /// Channel states
    channels: Vec<ChannelState>,
    /// Current mix view (Stream or Monitor)
    mix_mode: MixType,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    /// Create a new UI state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(UiStateInner {
                active_profile: "Default".to_string(),
                mix_mode: MixType::Stream,
                ..Default::default()
            })),
        }
    }

    /// Set the connection status.
    pub fn set_connected(&self, connected: bool) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        state.connected = connected;
        self.inner.store(Arc::new(state));
    }

    /// Check if connected to daemon.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.inner.load().connected
    }

    /// Set the device serial.
    pub fn set_device_serial(&self, serial: Option<String>) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        state.device_serial = serial;
        self.inner.store(Arc::new(state));
    }

    /// Get the device serial.
    #[must_use]
    pub fn device_serial(&self) -> Option<String> {
        self.inner.load().device_serial.clone()
    }

    /// Set the active profile.
    pub fn set_active_profile(&self, profile: String) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        state.active_profile = profile;
        self.inner.store(Arc::new(state));
    }

    /// Get the active profile.
    #[must_use]
    pub fn active_profile(&self) -> String {
        self.inner.load().active_profile.clone()
    }

    /// Update channels.
    pub fn set_channels(&self, channels: Vec<ChannelState>) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        state.channels = channels;
        self.inner.store(Arc::new(state));
    }

    /// Get channels.
    #[must_use]
    pub fn channels(&self) -> Vec<ChannelState> {
        self.inner.load().channels.clone()
    }

    /// Set the mix mode.
    pub fn set_mix_mode(&self, mode: MixType) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        state.mix_mode = mode;
        self.inner.store(Arc::new(state));
    }

    /// Get the mix mode.
    #[must_use]
    pub fn mix_mode(&self) -> MixType {
        self.inner.load().mix_mode
    }

    /// Update a channel's volume.
    pub fn update_channel_volume(&self, channel: &str, mix: MixType, volume: f32) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        if let Some(ch) = state.channels.iter_mut().find(|c| c.config.name == channel) {
            match mix {
                MixType::Stream => ch.stream_volume = volume,
                MixType::Monitor => ch.monitor_volume = volume,
            }
        }
        self.inner.store(Arc::new(state));
    }

    /// Update a channel's mute state.
    pub fn update_channel_mute(&self, channel: &str, mix: MixType, muted: bool) {
        let guard = self.inner.load();
        let mut state: UiStateInner = (**guard).clone();
        if let Some(ch) = state.channels.iter_mut().find(|c| c.config.name == channel) {
            match mix {
                MixType::Stream => ch.stream_muted = muted,
                MixType::Monitor => ch.monitor_muted = muted,
            }
        }
        self.inner.store(Arc::new(state));
    }
}
