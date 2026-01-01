//! Rust-Qt bridge definitions using cxx-qt.
//!
//! This module contains the cxx-qt bridge that exposes Rust types to QML.

use std::pin::Pin;
use std::sync::Arc;

use cxx_qt_lib::QString;
use tokio::sync::mpsc;
use tracing::debug;

use undertone_core::mixer::MixType;

use crate::state::UiState;

/// Channel data for QML model.
#[derive(Clone, Default)]
pub struct ChannelData {
    pub name: String,
    pub display_name: String,
    pub stream_volume: f32,
    pub stream_muted: bool,
    pub monitor_volume: f32,
    pub monitor_muted: bool,
    pub level_left: f32,
    pub level_right: f32,
}

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, connected)]
        #[qproperty(QString, device_serial)]
        #[qproperty(QString, active_profile)]
        #[qproperty(i32, mix_mode)]
        #[qproperty(i32, channel_count)]
        type UndertoneController = super::UndertoneControllerRust;

        /// Change the mix mode (0 = Stream, 1 = Monitor).
        #[qinvokable]
        fn change_mix_mode(self: Pin<&mut UndertoneController>, mode: i32);

        /// Set volume for a channel.
        #[qinvokable]
        fn set_channel_volume(self: Pin<&mut UndertoneController>, channel: QString, volume: f32);

        /// Toggle mute for a channel.
        #[qinvokable]
        fn toggle_channel_mute(self: Pin<&mut UndertoneController>, channel: QString);

        /// Get channel name by index.
        #[qinvokable]
        fn channel_name(self: &UndertoneController, index: i32) -> QString;

        /// Get channel display name by index.
        #[qinvokable]
        fn channel_display_name(self: &UndertoneController, index: i32) -> QString;

        /// Get channel volume by index (uses current mix mode).
        #[qinvokable]
        fn channel_volume(self: &UndertoneController, index: i32) -> f32;

        /// Get channel muted by index (uses current mix mode).
        #[qinvokable]
        fn channel_muted(self: &UndertoneController, index: i32) -> bool;

        /// Request state refresh from daemon.
        #[qinvokable]
        fn refresh(self: Pin<&mut UndertoneController>);
    }
}

/// Rust backing struct for UndertoneController QObject.
pub struct UndertoneControllerRust {
    connected: bool,
    device_serial: QString,
    active_profile: QString,
    mix_mode: i32,
    channel_count: i32,

    // Internal state
    channels: Vec<ChannelData>,
    state: Option<Arc<UiState>>,
    command_tx: Option<mpsc::Sender<UiCommand>>,
}

impl Default for UndertoneControllerRust {
    fn default() -> Self {
        Self {
            connected: false,
            device_serial: QString::from(""),
            active_profile: QString::from("Default"),
            mix_mode: 0,
            channel_count: 0,
            channels: Vec::new(),
            state: None,
            command_tx: None,
        }
    }
}

/// Commands sent to the async handler.
pub enum UiCommand {
    SetMixMode(MixType),
    SetVolume { channel: String, volume: f32 },
    ToggleMute { channel: String },
    Refresh,
}

impl ffi::UndertoneController {
    /// Change the mix mode.
    fn change_mix_mode(mut self: Pin<&mut Self>, mode: i32) {
        let mix = if mode == 0 {
            MixType::Stream
        } else {
            MixType::Monitor
        };
        debug!(?mix, "Changing mix mode");

        self.as_mut().set_mix_mode(mode);

        if let Some(tx) = &self.command_tx {
            let _ = tx.try_send(UiCommand::SetMixMode(mix));
        }
    }

    /// Set volume for a channel.
    fn set_channel_volume(self: Pin<&mut Self>, channel: QString, volume: f32) {
        let channel_name = channel.to_string();
        debug!(channel = %channel_name, volume, "Setting channel volume");

        if let Some(tx) = &self.command_tx {
            let _ = tx.try_send(UiCommand::SetVolume {
                channel: channel_name,
                volume,
            });
        }
    }

    /// Toggle mute for a channel.
    fn toggle_channel_mute(self: Pin<&mut Self>, channel: QString) {
        let channel_name = channel.to_string();
        debug!(channel = %channel_name, "Toggling channel mute");

        if let Some(tx) = &self.command_tx {
            let _ = tx.try_send(UiCommand::ToggleMute {
                channel: channel_name,
            });
        }
    }

    /// Get channel name by index.
    fn channel_name(&self, index: i32) -> QString {
        self.channels
            .get(index as usize)
            .map(|c| QString::from(&c.name))
            .unwrap_or_default()
    }

    /// Get channel display name by index.
    fn channel_display_name(&self, index: i32) -> QString {
        self.channels
            .get(index as usize)
            .map(|c| QString::from(&c.display_name))
            .unwrap_or_default()
    }

    /// Get channel volume by index.
    fn channel_volume(&self, index: i32) -> f32 {
        self.channels
            .get(index as usize)
            .map(|c| {
                if self.mix_mode == 0 {
                    c.stream_volume
                } else {
                    c.monitor_volume
                }
            })
            .unwrap_or(1.0)
    }

    /// Get channel muted by index.
    fn channel_muted(&self, index: i32) -> bool {
        self.channels
            .get(index as usize)
            .map(|c| {
                if self.mix_mode == 0 {
                    c.stream_muted
                } else {
                    c.monitor_muted
                }
            })
            .unwrap_or(false)
    }

    /// Request state refresh.
    fn refresh(self: Pin<&mut Self>) {
        debug!("Requesting state refresh");

        if let Some(tx) = &self.command_tx {
            let _ = tx.try_send(UiCommand::Refresh);
        }
    }
}

impl UndertoneControllerRust {
    /// Initialize the controller with state and command channel.
    pub fn init(&mut self, state: Arc<UiState>, command_tx: mpsc::Sender<UiCommand>) {
        self.state = Some(state);
        self.command_tx = Some(command_tx);
    }

    /// Update the connected status.
    pub fn update_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    /// Update the device serial.
    pub fn update_device_serial(&mut self, serial: Option<String>) {
        self.device_serial = QString::from(serial.as_deref().unwrap_or(""));
    }

    /// Update channels from state.
    pub fn update_channels(&mut self, channels: Vec<ChannelData>) {
        self.channel_count = channels.len() as i32;
        self.channels = channels;
    }
}
