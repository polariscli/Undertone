//! Request handling for the IPC server.

use serde_json::{json, Value};
use tracing::{debug, info};

use undertone_core::command::Command;
use undertone_core::state::StateSnapshot;
use undertone_ipc::messages::{ErrorInfo, Method};

/// Result of handling a request: response value and optional command.
pub struct HandleResult {
    pub response: Result<Value, ErrorInfo>,
    pub command: Option<Command>,
}

impl HandleResult {
    fn ok(value: Value) -> Self {
        Self {
            response: Ok(value),
            command: None,
        }
    }

    fn ok_with_command(value: Value, command: Command) -> Self {
        Self {
            response: Ok(value),
            command: Some(command),
        }
    }

    fn err(error: ErrorInfo) -> Self {
        Self {
            response: Err(error),
            command: None,
        }
    }
}

/// Handle an IPC request and return a response value with optional command.
pub fn handle_request(method: &Method, state: &StateSnapshot) -> HandleResult {
    match method {
        Method::GetState => HandleResult::ok(serde_json::to_value(state).unwrap_or(json!({}))),

        Method::GetChannels => {
            HandleResult::ok(serde_json::to_value(&state.channels).unwrap_or(json!([])))
        }

        Method::GetChannel { name } => {
            if let Some(ch) = state.channels.iter().find(|c| &c.config.name == name) {
                HandleResult::ok(serde_json::to_value(ch).unwrap_or(json!({})))
            } else {
                HandleResult::err(ErrorInfo::new(404, format!("Channel not found: {name}")))
            }
        }

        Method::GetApps => {
            HandleResult::ok(serde_json::to_value(&state.app_routes).unwrap_or(json!([])))
        }

        Method::GetProfiles => {
            // TODO: Load profiles from database
            HandleResult::ok(json!([{"name": "Default", "is_default": true}]))
        }

        Method::GetProfile { name } => {
            // TODO: Load profile from database
            if name == "Default" {
                HandleResult::ok(json!({"name": "Default", "is_default": true}))
            } else {
                HandleResult::err(ErrorInfo::new(404, format!("Profile not found: {name}")))
            }
        }

        Method::GetDeviceStatus => HandleResult::ok(json!({
            "connected": state.device_connected,
            "serial": state.device_serial,
        })),

        Method::GetDiagnostics => HandleResult::ok(json!({
            "state": format!("{:?}", state.state),
            "created_nodes": state.created_nodes.len(),
            "created_links": state.created_links.len(),
        })),

        Method::SetChannelVolume {
            channel,
            mix,
            volume,
        } => {
            // Validate channel exists
            if !state.channels.iter().any(|c| &c.config.name == channel) {
                return HandleResult::err(ErrorInfo::new(
                    404,
                    format!("Channel not found: {channel}"),
                ));
            }
            // Clamp volume to valid range
            let volume = volume.clamp(0.0, 1.0);
            debug!(?channel, ?mix, volume, "Setting channel volume");
            HandleResult::ok_with_command(
                json!({"success": true, "volume": volume}),
                Command::SetChannelVolume {
                    channel: channel.clone(),
                    mix: *mix,
                    volume,
                },
            )
        }

        Method::SetChannelMute {
            channel,
            mix,
            muted,
        } => {
            // Validate channel exists
            if !state.channels.iter().any(|c| &c.config.name == channel) {
                return HandleResult::err(ErrorInfo::new(
                    404,
                    format!("Channel not found: {channel}"),
                ));
            }
            debug!(?channel, ?mix, muted, "Setting channel mute");
            HandleResult::ok_with_command(
                json!({"success": true, "muted": muted}),
                Command::SetChannelMute {
                    channel: channel.clone(),
                    mix: *mix,
                    muted: *muted,
                },
            )
        }

        Method::SetAppRoute {
            app_pattern,
            channel,
        } => {
            // Validate channel exists
            if !state.channels.iter().any(|c| &c.config.name == channel) {
                return HandleResult::err(ErrorInfo::new(
                    404,
                    format!("Channel not found: {channel}"),
                ));
            }
            info!(?app_pattern, ?channel, "Setting app route");
            HandleResult::ok_with_command(
                json!({"success": true}),
                Command::SetAppRoute {
                    app_pattern: app_pattern.clone(),
                    channel: channel.clone(),
                },
            )
        }

        Method::RemoveAppRoute { app_pattern } => {
            info!(?app_pattern, "Removing app route");
            HandleResult::ok_with_command(
                json!({"success": true}),
                Command::RemoveAppRoute {
                    app_pattern: app_pattern.clone(),
                },
            )
        }

        Method::SaveProfile { name } => {
            info!(?name, "Saving profile");
            HandleResult::ok_with_command(
                json!({"success": true}),
                Command::SaveProfile { name: name.clone() },
            )
        }

        Method::LoadProfile { name } => {
            info!(?name, "Loading profile");
            HandleResult::ok_with_command(
                json!({"success": true}),
                Command::LoadProfile { name: name.clone() },
            )
        }

        Method::DeleteProfile { name } => {
            info!(?name, "Deleting profile");
            HandleResult::ok_with_command(
                json!({"success": true}),
                Command::DeleteProfile { name: name.clone() },
            )
        }

        Method::SetMicGain { gain } => {
            let gain = gain.clamp(0.0, 1.0);
            debug!(gain, "Setting mic gain");
            HandleResult::ok_with_command(
                json!({"success": true, "gain": gain}),
                Command::SetMicGain { gain },
            )
        }

        Method::SetMicMute { muted } => {
            debug!(muted, "Setting mic mute");
            HandleResult::ok_with_command(
                json!({"success": true, "muted": muted}),
                Command::SetMicMute { muted: *muted },
            )
        }

        Method::Subscribe { events } => {
            debug!(?events, "Client subscribing to events");
            // Subscription handling is done in the IPC server
            HandleResult::ok(json!({"success": true}))
        }

        Method::Unsubscribe { events } => {
            debug!(?events, "Client unsubscribing from events");
            // Unsubscription handling is done in the IPC server
            HandleResult::ok(json!({"success": true}))
        }

        Method::Shutdown => {
            info!("Shutdown requested via IPC");
            HandleResult::ok_with_command(json!({"success": true}), Command::Shutdown)
        }

        Method::Reconcile => {
            info!("Reconciliation requested via IPC");
            HandleResult::ok_with_command(json!({"success": true}), Command::Reconcile)
        }
    }
}
