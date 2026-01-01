//! Rust-Qt bridge definitions using cxx-qt.
//!
//! This module contains the cxx-qt bridge that exposes Rust types to QML.

use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};

use cxx_qt_lib::QString;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use undertone_core::mixer::MixType;

use crate::ipc_handler::{IpcHandle, IpcUpdate};
use crate::state::UiState;

/// Global IPC handle for communicating with the daemon.
static IPC_HANDLE: OnceLock<Mutex<IpcHandle>> = OnceLock::new();

/// Global UI data cache updated by IPC handler.
static UI_DATA: OnceLock<Mutex<UiDataCache>> = OnceLock::new();

/// Cached UI data from daemon.
#[derive(Default)]
struct UiDataCache {
    channels: Vec<ChannelData>,
    apps: Vec<AppData>,
    profiles: Vec<ProfileData>,
}

fn get_ui_data() -> &'static Mutex<UiDataCache> {
    UI_DATA.get_or_init(|| {
        Mutex::new(UiDataCache {
            channels: Vec::new(),
            apps: Vec::new(),
            profiles: vec![ProfileData { name: "Default".to_string(), is_default: true }],
        })
    })
}

/// Send a command to the daemon via the global IPC handle.
fn send_command(cmd: UiCommand) {
    if let Some(handle) = IPC_HANDLE.get() {
        if let Ok(guard) = handle.lock() {
            let _ = guard.command_sender().try_send(cmd);
        }
    }
}

/// Initialize the global IPC handle.
/// This should be called once before the QML engine starts.
pub fn init_ipc(state: Arc<UiState>) -> mpsc::Sender<UiCommand> {
    let handle = IpcHandle::start(state);
    let sender = handle.command_sender();
    IPC_HANDLE.set(Mutex::new(handle)).expect("IPC handle already initialized");
    info!("IPC handle initialized");
    sender
}

/// Channel data for QML model.
#[derive(Clone, Default, Debug)]
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

/// App data for QML model.
#[derive(Clone, Default, Debug)]
pub struct AppData {
    pub app_id: u32,
    pub name: String,
    pub binary_name: String,
    pub channel: String,
    pub is_persistent: bool,
}

/// Profile data for QML model.
#[derive(Clone, Default, Debug)]
pub struct ProfileData {
    pub name: String,
    pub is_default: bool,
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
        #[qproperty(i32, app_count)]
        #[qproperty(bool, mic_muted)]
        #[qproperty(f32, mic_gain)]
        #[qproperty(i32, profile_count)]
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

        // App routing methods

        /// Get app name by index.
        #[qinvokable]
        fn app_name(self: &UndertoneController, index: i32) -> QString;

        /// Get app binary name by index.
        #[qinvokable]
        fn app_binary(self: &UndertoneController, index: i32) -> QString;

        /// Get app's current channel by index.
        #[qinvokable]
        fn app_channel(self: &UndertoneController, index: i32) -> QString;

        /// Check if app route is persistent by index.
        #[qinvokable]
        fn app_persistent(self: &UndertoneController, index: i32) -> bool;

        /// Set app channel routing.
        #[qinvokable]
        fn set_app_channel(
            self: Pin<&mut UndertoneController>,
            app_pattern: QString,
            channel: QString,
        );

        /// Get list of available channel names (for dropdown).
        #[qinvokable]
        fn available_channels(self: &UndertoneController) -> QString;

        // Device control methods

        /// Set microphone gain (0.0 - 1.0).
        #[qinvokable]
        fn set_mic_gain_value(self: Pin<&mut UndertoneController>, gain: f32);

        /// Toggle microphone mute.
        #[qinvokable]
        fn toggle_mic_mute(self: Pin<&mut UndertoneController>);

        // Profile methods

        /// Get profile name by index.
        #[qinvokable]
        fn profile_name(self: &UndertoneController, index: i32) -> QString;

        /// Check if profile is default by index.
        #[qinvokable]
        fn profile_is_default(self: &UndertoneController, index: i32) -> bool;

        /// Save current state as a profile.
        #[qinvokable]
        fn save_profile(self: Pin<&mut UndertoneController>, name: QString);

        /// Load a saved profile.
        #[qinvokable]
        fn load_profile(self: Pin<&mut UndertoneController>, name: QString);

        /// Delete a profile.
        #[qinvokable]
        fn delete_profile(self: Pin<&mut UndertoneController>, name: QString);

        /// Poll for updates from the IPC handler.
        /// Call this periodically from a QML Timer.
        #[qinvokable]
        fn poll_updates(self: Pin<&mut UndertoneController>);

        /// Initialize the controller and connect to daemon.
        #[qinvokable]
        fn initialize(self: Pin<&mut UndertoneController>);
    }
}

/// Rust backing struct for UndertoneController QObject.
pub struct UndertoneControllerRust {
    connected: bool,
    device_serial: QString,
    active_profile: QString,
    mix_mode: i32,
    channel_count: i32,
    app_count: i32,
    mic_muted: bool,
    mic_gain: f32,
    profile_count: i32,
}

impl Default for UndertoneControllerRust {
    fn default() -> Self {
        Self {
            connected: false,
            device_serial: QString::from(""),
            active_profile: QString::from("Default"),
            mix_mode: 0,
            channel_count: 0,
            app_count: 0,
            mic_muted: false,
            mic_gain: 0.75,
            profile_count: 1,
        }
    }
}

/// Commands sent to the async handler.
pub enum UiCommand {
    SetMixMode(MixType),
    SetVolume { channel: String, volume: f32 },
    ToggleMute { channel: String },
    SetAppChannel { app_pattern: String, channel: String },
    SetMicGain { gain: f32 },
    ToggleMicMute,
    SaveProfile { name: String },
    LoadProfile { name: String },
    DeleteProfile { name: String },
    Refresh,
}

impl ffi::UndertoneController {
    /// Change the mix mode.
    fn change_mix_mode(mut self: Pin<&mut Self>, mode: i32) {
        let mix = if mode == 0 { MixType::Stream } else { MixType::Monitor };
        debug!(?mix, "Changing mix mode");

        self.as_mut().set_mix_mode(mode);
        send_command(UiCommand::SetMixMode(mix));
    }

    /// Set volume for a channel.
    fn set_channel_volume(self: Pin<&mut Self>, channel: QString, volume: f32) {
        let channel_name = channel.to_string();
        debug!(channel = %channel_name, volume, "Setting channel volume");

        send_command(UiCommand::SetVolume { channel: channel_name, volume });
    }

    /// Toggle mute for a channel.
    fn toggle_channel_mute(self: Pin<&mut Self>, channel: QString) {
        let channel_name = channel.to_string();
        debug!(channel = %channel_name, "Toggling channel mute");

        send_command(UiCommand::ToggleMute { channel: channel_name });
    }

    /// Get channel name by index.
    fn channel_name(&self, index: i32) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            cache.channels.get(index as usize).map(|c| QString::from(&c.name)).unwrap_or_default()
        } else {
            QString::default()
        }
    }

    /// Get channel display name by index.
    fn channel_display_name(&self, index: i32) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            cache
                .channels
                .get(index as usize)
                .map(|c| QString::from(&c.display_name))
                .unwrap_or_default()
        } else {
            QString::default()
        }
    }

    /// Get channel volume by index.
    fn channel_volume(&self, index: i32) -> f32 {
        if let Ok(cache) = get_ui_data().lock() {
            cache
                .channels
                .get(index as usize)
                .map(|c| if self.mix_mode == 0 { c.stream_volume } else { c.monitor_volume })
                .unwrap_or(1.0)
        } else {
            1.0
        }
    }

    /// Get channel muted by index.
    fn channel_muted(&self, index: i32) -> bool {
        if let Ok(cache) = get_ui_data().lock() {
            cache
                .channels
                .get(index as usize)
                .map(|c| if self.mix_mode == 0 { c.stream_muted } else { c.monitor_muted })
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Request state refresh.
    fn refresh(self: Pin<&mut Self>) {
        debug!("Requesting state refresh");
        send_command(UiCommand::Refresh);
    }

    /// Get app name by index.
    fn app_name(&self, index: i32) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            cache.apps.get(index as usize).map(|a| QString::from(&a.name)).unwrap_or_default()
        } else {
            QString::default()
        }
    }

    /// Get app binary name by index.
    fn app_binary(&self, index: i32) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            cache
                .apps
                .get(index as usize)
                .map(|a| QString::from(&a.binary_name))
                .unwrap_or_default()
        } else {
            QString::default()
        }
    }

    /// Get app's current channel by index.
    fn app_channel(&self, index: i32) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            cache.apps.get(index as usize).map(|a| QString::from(&a.channel)).unwrap_or_default()
        } else {
            QString::default()
        }
    }

    /// Check if app route is persistent by index.
    fn app_persistent(&self, index: i32) -> bool {
        if let Ok(cache) = get_ui_data().lock() {
            cache.apps.get(index as usize).is_some_and(|a| a.is_persistent)
        } else {
            false
        }
    }

    /// Set app channel routing.
    fn set_app_channel(self: Pin<&mut Self>, app_pattern: QString, channel: QString) {
        let pattern = app_pattern.to_string();
        let channel_name = channel.to_string();
        debug!(app = %pattern, channel = %channel_name, "Setting app channel");

        send_command(UiCommand::SetAppChannel { app_pattern: pattern, channel: channel_name });
    }

    /// Get list of available channel names (comma-separated for QML).
    fn available_channels(&self) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            let names: Vec<&str> = cache.channels.iter().map(|c| c.name.as_str()).collect();
            QString::from(names.join(",").as_str())
        } else {
            QString::default()
        }
    }

    /// Set microphone gain.
    fn set_mic_gain_value(mut self: Pin<&mut Self>, gain: f32) {
        debug!(gain, "Setting mic gain");
        self.as_mut().set_mic_gain(gain);
        send_command(UiCommand::SetMicGain { gain });
    }

    /// Toggle microphone mute.
    fn toggle_mic_mute(mut self: Pin<&mut Self>) {
        let new_muted = !self.mic_muted;
        debug!(muted = new_muted, "Toggling mic mute");
        self.as_mut().set_mic_muted(new_muted);
        send_command(UiCommand::ToggleMicMute);
    }

    /// Get profile name by index.
    fn profile_name(&self, index: i32) -> QString {
        if let Ok(cache) = get_ui_data().lock() {
            cache.profiles.get(index as usize).map(|p| QString::from(&p.name)).unwrap_or_default()
        } else {
            QString::default()
        }
    }

    /// Check if profile is default by index.
    fn profile_is_default(&self, index: i32) -> bool {
        if let Ok(cache) = get_ui_data().lock() {
            cache.profiles.get(index as usize).is_some_and(|p| p.is_default)
        } else {
            false
        }
    }

    /// Save current state as a profile.
    fn save_profile(self: Pin<&mut Self>, name: QString) {
        let profile_name = name.to_string();
        debug!(name = %profile_name, "Saving profile");
        send_command(UiCommand::SaveProfile { name: profile_name });
    }

    /// Load a saved profile.
    fn load_profile(mut self: Pin<&mut Self>, name: QString) {
        let profile_name = name.to_string();
        debug!(name = %profile_name, "Loading profile");

        // Update active profile immediately for UI feedback
        self.as_mut().set_active_profile(name);
        send_command(UiCommand::LoadProfile { name: profile_name });
    }

    /// Delete a profile.
    fn delete_profile(self: Pin<&mut Self>, name: QString) {
        let profile_name = name.to_string();
        debug!(name = %profile_name, "Deleting profile");
        send_command(UiCommand::DeleteProfile { name: profile_name });
    }

    /// Initialize the controller and connect to daemon.
    fn initialize(self: Pin<&mut Self>) {
        // Nothing to do here - IPC is initialized globally in app.rs
        // This is just a hook for QML to call on startup
        info!("Controller initialized");
    }

    /// Poll for updates from the IPC handler.
    fn poll_updates(mut self: Pin<&mut Self>) {
        // Try to get updates from the IPC handler
        if let Some(handle) = IPC_HANDLE.get() {
            if let Ok(mut guard) = handle.lock() {
                while let Some(update) = guard.try_recv_update() {
                    self.as_mut().apply_update(update);
                }
            }
        }
    }
}

impl ffi::UndertoneController {
    /// Apply an IPC update to the controller state.
    fn apply_update(mut self: Pin<&mut Self>, update: IpcUpdate) {
        match update {
            IpcUpdate::Connected => {
                info!("Connected to daemon");
                self.as_mut().set_connected(true);
            }
            IpcUpdate::Disconnected => {
                warn!("Disconnected from daemon");
                self.as_mut().set_connected(false);
            }
            IpcUpdate::StateUpdated {
                channels,
                apps,
                profiles,
                device_connected,
                device_serial,
                active_profile,
            } => {
                debug!(
                    channels = channels.len(),
                    apps = apps.len(),
                    profiles = profiles.len(),
                    "State updated from daemon"
                );

                // Update simple properties
                self.as_mut().set_connected(device_connected);
                self.as_mut()
                    .set_device_serial(QString::from(device_serial.as_deref().unwrap_or("")));
                self.as_mut().set_active_profile(QString::from(active_profile.as_str()));

                // Update global cache for vector data
                if let Ok(mut cache) = get_ui_data().lock() {
                    cache.channels = channels;
                    cache.apps = apps;
                    cache.profiles = profiles;

                    // Update counts to trigger QML refresh
                    self.as_mut().set_channel_count(cache.channels.len() as i32);
                    self.as_mut().set_app_count(cache.apps.len() as i32);
                    self.as_mut().set_profile_count(cache.profiles.len() as i32);
                }
            }
            IpcUpdate::ChannelVolumeChanged { channel, mix, volume } => {
                debug!(channel = %channel, ?mix, volume, "Channel volume changed");
                if let Ok(mut cache) = get_ui_data().lock() {
                    if let Some(ch) = cache.channels.iter_mut().find(|c| c.name == channel) {
                        match mix {
                            MixType::Stream => ch.stream_volume = volume,
                            MixType::Monitor => ch.monitor_volume = volume,
                        }
                    }
                    // Trigger UI refresh
                    let count = cache.channels.len() as i32;
                    self.as_mut().set_channel_count(count);
                }
            }
            IpcUpdate::ChannelMuteChanged { channel, mix, muted } => {
                debug!(channel = %channel, ?mix, muted, "Channel mute changed");
                if let Ok(mut cache) = get_ui_data().lock() {
                    if let Some(ch) = cache.channels.iter_mut().find(|c| c.name == channel) {
                        match mix {
                            MixType::Stream => ch.stream_muted = muted,
                            MixType::Monitor => ch.monitor_muted = muted,
                        }
                    }
                    let count = cache.channels.len() as i32;
                    self.as_mut().set_channel_count(count);
                }
            }
            IpcUpdate::DeviceConnected { serial } => {
                info!(?serial, "Device connected");
                self.as_mut().set_connected(true);
                self.as_mut().set_device_serial(QString::from(serial.as_deref().unwrap_or("")));
            }
            IpcUpdate::DeviceDisconnected => {
                info!("Device disconnected");
                self.as_mut().set_connected(false);
                self.as_mut().set_device_serial(QString::from(""));
            }
            IpcUpdate::Error(msg) => {
                warn!(error = %msg, "IPC error");
            }
        }
    }
}
