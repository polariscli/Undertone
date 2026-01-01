//! IPC handler for communicating with the Undertone daemon.
//!
//! This module runs a tokio runtime in a background thread and handles
//! all communication with the daemon, bridging between the async IPC client
//! and the synchronous Qt event loop.

use std::sync::Arc;
use std::thread;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use undertone_ipc::client::IpcClient;
use undertone_ipc::events::{
    ChannelMuteChangedData, ChannelVolumeChangedData, DeviceConnectedData, Event, EventType,
};
use undertone_ipc::messages::Method;

use crate::bridge::{AppData, ChannelData, ProfileData, UiCommand};
use crate::state::UiState;
use undertone_core::mixer::MixType;

/// Messages sent from the IPC handler back to the UI.
#[derive(Debug)]
pub enum IpcUpdate {
    Connected,
    Disconnected,
    StateUpdated {
        channels: Vec<ChannelData>,
        apps: Vec<AppData>,
        profiles: Vec<ProfileData>,
        device_connected: bool,
        device_serial: Option<String>,
        active_profile: String,
    },
    ChannelVolumeChanged {
        channel: String,
        mix: MixType,
        volume: f32,
    },
    ChannelMuteChanged {
        channel: String,
        mix: MixType,
        muted: bool,
    },
    DeviceConnected {
        serial: Option<String>,
    },
    DeviceDisconnected,
    Error(String),
}

/// Handle for communicating with the IPC handler thread.
pub struct IpcHandle {
    command_tx: mpsc::Sender<UiCommand>,
    update_rx: mpsc::Receiver<IpcUpdate>,
}

impl std::fmt::Debug for IpcHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpcHandle").finish_non_exhaustive()
    }
}

impl IpcHandle {
    /// Start the IPC handler in a background thread.
    #[must_use]
    pub fn start(state: Arc<UiState>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(64);
        let (update_tx, update_rx) = mpsc::channel(64);

        thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(run_ipc_loop(state, command_rx, update_tx));
        });

        Self {
            command_tx,
            update_rx,
        }
    }

    /// Send a command to the daemon.
    pub fn send_command(&self, command: UiCommand) {
        let _ = self.command_tx.blocking_send(command);
    }

    /// Try to receive an update without blocking.
    pub fn try_recv_update(&mut self) -> Option<IpcUpdate> {
        self.update_rx.try_recv().ok()
    }

    /// Get the command sender for cloning.
    pub fn command_sender(&self) -> mpsc::Sender<UiCommand> {
        self.command_tx.clone()
    }
}

async fn run_ipc_loop(
    _state: Arc<UiState>,
    mut command_rx: mpsc::Receiver<UiCommand>,
    update_tx: mpsc::Sender<IpcUpdate>,
) {
    info!("IPC handler starting");

    loop {
        // Try to connect to the daemon
        match IpcClient::connect_default().await {
            Ok(mut client) => {
                info!("Connected to daemon");
                let _ = update_tx.send(IpcUpdate::Connected).await;

                // Subscribe to events
                if let Err(e) = client
                    .request(Method::Subscribe {
                        events: vec![
                            "channel_volume_changed".to_string(),
                            "channel_mute_changed".to_string(),
                            "device_connected".to_string(),
                            "device_disconnected".to_string(),
                            "app_discovered".to_string(),
                            "app_removed".to_string(),
                            "profile_changed".to_string(),
                        ],
                    })
                    .await
                {
                    warn!(error = %e, "Failed to subscribe to events");
                }

                // Request initial state
                if let Ok(response) = client.request(Method::GetState).await {
                    if let Ok(value) = response.result {
                        if let Some(state_data) = parse_state_response(&value) {
                            let _ = update_tx.send(state_data).await;
                        }
                    }
                }

                // Main loop: handle commands and events
                loop {
                    tokio::select! {
                        // Handle commands from UI
                        Some(cmd) = command_rx.recv() => {
                            if let Some(method) = command_to_method(cmd) {
                                if let Err(e) = client.request(method).await {
                                    error!(error = %e, "Request failed");
                                }
                            }
                        }

                        // Handle events from daemon
                        Some(event) = client.events().recv() => {
                            if let Some(update) = event_to_update(event) {
                                let _ = update_tx.send(update).await;
                            }
                        }

                        else => {
                            // Connection lost
                            break;
                        }
                    }
                }

                warn!("Connection to daemon lost");
                let _ = update_tx.send(IpcUpdate::Disconnected).await;
            }
            Err(e) => {
                debug!(error = %e, "Failed to connect to daemon, retrying in 2s");
            }
        }

        // Wait before reconnecting
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

fn command_to_method(cmd: UiCommand) -> Option<Method> {
    match cmd {
        UiCommand::SetMixMode(_mode) => {
            // Mix mode is UI-only, not sent to daemon
            None
        }
        UiCommand::SetVolume { channel, volume } => Some(Method::SetChannelVolume {
            channel,
            mix: MixType::Stream, // TODO: Use current mix mode
            volume,
        }),
        UiCommand::ToggleMute { channel } => Some(Method::SetChannelMute {
            channel,
            mix: MixType::Stream,
            muted: true, // TODO: Toggle based on current state
        }),
        UiCommand::SetAppChannel {
            app_pattern,
            channel,
        } => Some(Method::SetAppRoute {
            app_pattern,
            channel,
        }),
        UiCommand::SetMicGain { gain } => Some(Method::SetMicGain { gain }),
        UiCommand::ToggleMicMute => Some(Method::SetMicMute { muted: true }), // TODO: Toggle
        UiCommand::SaveProfile { name } => Some(Method::SaveProfile { name }),
        UiCommand::LoadProfile { name } => Some(Method::LoadProfile { name }),
        UiCommand::DeleteProfile { name } => Some(Method::DeleteProfile { name }),
        UiCommand::Refresh => Some(Method::GetState),
    }
}

fn event_to_update(event: Event) -> Option<IpcUpdate> {
    match event.event {
        EventType::ChannelVolumeChanged => {
            // Parse the event data
            if let Ok(data) = serde_json::from_value::<ChannelVolumeChangedData>(event.data) {
                Some(IpcUpdate::ChannelVolumeChanged {
                    channel: data.channel,
                    mix: data.mix,
                    volume: data.volume,
                })
            } else {
                warn!("Failed to parse ChannelVolumeChanged event");
                None
            }
        }
        EventType::ChannelMuteChanged => {
            if let Ok(data) = serde_json::from_value::<ChannelMuteChangedData>(event.data) {
                Some(IpcUpdate::ChannelMuteChanged {
                    channel: data.channel,
                    mix: data.mix,
                    muted: data.muted,
                })
            } else {
                warn!("Failed to parse ChannelMuteChanged event");
                None
            }
        }
        EventType::DeviceConnected => {
            if let Ok(data) = serde_json::from_value::<DeviceConnectedData>(event.data) {
                Some(IpcUpdate::DeviceConnected {
                    serial: Some(data.serial),
                })
            } else {
                Some(IpcUpdate::DeviceConnected { serial: None })
            }
        }
        EventType::DeviceDisconnected => Some(IpcUpdate::DeviceDisconnected),
        _ => None,
    }
}

fn parse_state_response(value: &serde_json::Value) -> Option<IpcUpdate> {
    use serde_json::Value;

    // Parse channels - daemon format has config.name, config.display_name
    let channels = value
        .get("channels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|ch| {
                    // Channel config is nested
                    let config = ch.get("config")?;
                    Some(ChannelData {
                        name: config.get("name")?.as_str()?.to_string(),
                        display_name: config
                            .get("display_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        stream_volume: ch
                            .get("stream_volume")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0) as f32,
                        stream_muted: ch
                            .get("stream_muted")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        monitor_volume: ch
                            .get("monitor_volume")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0) as f32,
                        monitor_muted: ch
                            .get("monitor_muted")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        level_left: ch
                            .get("level_left")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32,
                        level_right: ch
                            .get("level_right")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse app routes
    let apps = value
        .get("app_routes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|app| {
                    Some(AppData {
                        app_id: app.get("app_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        name: app
                            .get("app_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        binary_name: app
                            .get("binary_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        channel: app
                            .get("channel")
                            .and_then(|v| v.as_str())
                            .unwrap_or("system")
                            .to_string(),
                        is_persistent: app
                            .get("is_persistent")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Daemon doesn't send profiles array, just active_profile
    // Create default profile list
    let active_profile = value
        .get("active_profile")
        .and_then(|v: &Value| v.as_str())
        .unwrap_or("Default")
        .to_string();

    let profiles = vec![ProfileData {
        name: active_profile.clone(),
        is_default: active_profile == "Default",
    }];

    let device_connected = value
        .get("device_connected")
        .and_then(|v: &Value| v.as_bool())
        .unwrap_or(false);

    let device_serial = value
        .get("device_serial")
        .and_then(|v: &Value| v.as_str())
        .map(String::from);

    Some(IpcUpdate::StateUpdated {
        channels,
        apps,
        profiles,
        device_connected,
        device_serial,
        active_profile,
    })
}
