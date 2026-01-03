//! ALSA fallback for mic control.
//!
//! When direct HID control is not available, we can use ALSA mixer
//! controls to adjust the Wave:3 microphone gain and mute state.

use tracing::{debug, warn};

use crate::error::{HidError, HidResult};

/// ALSA-based microphone control.
pub struct AlsaMicControl {
    card_name: String,
}

impl AlsaMicControl {
    /// Create a new ALSA mic control for the given card.
    #[must_use]
    pub fn new(card_name: String) -> Self {
        Self { card_name }
    }

    /// Set the microphone volume.
    ///
    /// # Arguments
    /// * `volume` - Volume level from 0.0 to 1.0
    ///
    /// # Errors
    /// Returns an error if the ALSA control cannot be accessed.
    pub fn set_volume(&self, volume: f32) -> HidResult<()> {
        let volume_percent = (volume.clamp(0.0, 1.0) * 100.0) as u32;

        // Use amixer as a simple approach
        // In production, we'd use alsa-rs directly
        let output = std::process::Command::new("amixer")
            .args(["-c", &self.card_name, "sset", "Mic", &format!("{volume_percent}%")])
            .output()
            .map_err(|e| HidError::AlsaError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HidError::AlsaError(format!("amixer failed: {stderr}")));
        }

        debug!(volume_percent, "Mic volume set via ALSA");
        Ok(())
    }

    /// Get the current microphone volume.
    ///
    /// # Errors
    /// Returns an error if the ALSA control cannot be accessed.
    pub fn get_volume(&self) -> HidResult<f32> {
        let output = std::process::Command::new("amixer")
            .args(["-c", &self.card_name, "sget", "Mic"])
            .output()
            .map_err(|e| HidError::AlsaError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HidError::AlsaError(format!("amixer failed: {stderr}")));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse output like "[50%]"
        for part in stdout.split_whitespace() {
            if part.starts_with('[') && part.ends_with("%]") {
                let percent_str = part.trim_start_matches('[').trim_end_matches("%]");
                if let Ok(percent) = percent_str.parse::<u32>() {
                    return Ok(percent as f32 / 100.0);
                }
            }
        }

        warn!("Could not parse ALSA volume output");
        Ok(1.0) // Default to full volume if parsing fails
    }

    /// Set the microphone mute state.
    ///
    /// # Errors
    /// Returns an error if the ALSA control cannot be accessed.
    pub fn set_mute(&self, muted: bool) -> HidResult<()> {
        let state = if muted { "mute" } else { "unmute" };

        let output = std::process::Command::new("amixer")
            .args(["-c", &self.card_name, "sset", "Mic", state])
            .output()
            .map_err(|e| HidError::AlsaError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HidError::AlsaError(format!("amixer failed: {stderr}")));
        }

        debug!(muted, "Mic mute set via ALSA");
        Ok(())
    }

    /// Get the current microphone mute state.
    ///
    /// # Errors
    /// Returns an error if the ALSA control cannot be accessed.
    pub fn get_mute(&self) -> HidResult<bool> {
        let output = std::process::Command::new("amixer")
            .args(["-c", &self.card_name, "sget", "Mic"])
            .output()
            .map_err(|e| HidError::AlsaError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HidError::AlsaError(format!("amixer failed: {stderr}")));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Look for [on] or [off]
        if stdout.contains("[off]") {
            return Ok(true);
        }
        if stdout.contains("[on]") {
            return Ok(false);
        }

        warn!("Could not parse ALSA mute state");
        Ok(false) // Default to unmuted if parsing fails
    }
}
