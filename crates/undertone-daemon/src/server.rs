//! Request handling for the IPC server.

use serde_json::{json, Value};
use tracing::{debug, info};

use undertone_core::mixer::MixType;
use undertone_core::state::StateSnapshot;
use undertone_ipc::messages::{ErrorInfo, Method};

/// Handle an IPC request and return a response value.
pub fn handle_request(method: &Method, state: &StateSnapshot) -> Result<Value, ErrorInfo> {
    match method {
        Method::GetState => {
            Ok(serde_json::to_value(state).unwrap_or(json!({})))
        }

        Method::GetChannels => {
            Ok(serde_json::to_value(&state.channels).unwrap_or(json!([])))
        }

        Method::GetChannel { name } => {
            if let Some(ch) = state.channels.iter().find(|c| &c.config.name == name) {
                Ok(serde_json::to_value(ch).unwrap_or(json!({})))
            } else {
                Err(ErrorInfo::new(404, format!("Channel not found: {name}")))
            }
        }

        Method::GetApps => {
            Ok(serde_json::to_value(&state.app_routes).unwrap_or(json!([])))
        }

        Method::GetProfiles => {
            // TODO: Load profiles from database
            Ok(json!([{"name": "Default", "is_default": true}]))
        }

        Method::GetProfile { name } => {
            // TODO: Load profile from database
            if name == "Default" {
                Ok(json!({"name": "Default", "is_default": true}))
            } else {
                Err(ErrorInfo::new(404, format!("Profile not found: {name}")))
            }
        }

        Method::GetDeviceStatus => {
            Ok(json!({
                "connected": state.device_connected,
                "serial": state.device_serial,
            }))
        }

        Method::GetDiagnostics => {
            Ok(json!({
                "state": format!("{:?}", state.state),
                "created_nodes": state.created_nodes.len(),
                "created_links": state.created_links.len(),
            }))
        }

        Method::SetChannelVolume { channel, mix, volume } => {
            debug!(?channel, ?mix, volume, "Setting channel volume");
            // TODO: Apply volume change to PipeWire node
            Ok(json!({"success": true}))
        }

        Method::SetChannelMute { channel, mix, muted } => {
            debug!(?channel, ?mix, muted, "Setting channel mute");
            // TODO: Apply mute change to PipeWire node
            Ok(json!({"success": true}))
        }

        Method::SetAppRoute { app_pattern, channel } => {
            info!(?app_pattern, ?channel, "Setting app route");
            // TODO: Save route to database and apply
            Ok(json!({"success": true}))
        }

        Method::RemoveAppRoute { app_pattern } => {
            info!(?app_pattern, "Removing app route");
            // TODO: Remove route from database
            Ok(json!({"success": true}))
        }

        Method::SaveProfile { name } => {
            info!(?name, "Saving profile");
            // TODO: Save current state as profile
            Ok(json!({"success": true}))
        }

        Method::LoadProfile { name } => {
            info!(?name, "Loading profile");
            // TODO: Load profile and apply
            Ok(json!({"success": true}))
        }

        Method::DeleteProfile { name } => {
            info!(?name, "Deleting profile");
            // TODO: Delete profile from database
            Ok(json!({"success": true}))
        }

        Method::SetMicGain { gain } => {
            debug!(gain, "Setting mic gain");
            // TODO: Apply via ALSA or HID
            Ok(json!({"success": true}))
        }

        Method::SetMicMute { muted } => {
            debug!(muted, "Setting mic mute");
            // TODO: Apply via ALSA or HID
            Ok(json!({"success": true}))
        }

        Method::Subscribe { events } => {
            debug!(?events, "Client subscribing to events");
            Ok(json!({"success": true}))
        }

        Method::Unsubscribe { events } => {
            debug!(?events, "Client unsubscribing from events");
            Ok(json!({"success": true}))
        }

        Method::Shutdown => {
            info!("Shutdown requested via IPC");
            // The main loop will handle this
            Ok(json!({"success": true}))
        }

        Method::Reconcile => {
            info!("Reconciliation requested via IPC");
            // TODO: Trigger reconciliation
            Ok(json!({"success": true}))
        }
    }
}
