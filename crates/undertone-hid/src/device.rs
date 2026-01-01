//! Wave:3 device detection and control.

use tracing::{debug, info, warn};

use crate::error::{HidError, HidResult};

/// Elgato USB Vendor ID
pub const ELGATO_VID: u16 = 0x0fd9;
/// Wave:3 USB Product ID
pub const WAVE3_PID: u16 = 0x0070;
/// Vendor-specific control interface
pub const CONTROL_INTERFACE: u8 = 3;

/// Represents a connected Wave:3 device.
pub struct Wave3Device {
    /// Device serial number
    serial: String,
    /// ALSA card name for fallback control
    alsa_card: Option<String>,
}

impl Wave3Device {
    /// Attempt to detect a connected Wave:3 device.
    ///
    /// # Errors
    /// Returns an error if no device is found or detection fails.
    pub fn detect() -> HidResult<Option<Self>> {
        // Try to find Wave:3 via ALSA first (more reliable)
        if let Some(card) = Self::find_alsa_card()? {
            info!(card = %card, "Wave:3 detected via ALSA");

            // Try to get serial from ALSA card info
            let serial = Self::get_serial_from_alsa(&card).unwrap_or_else(|| "unknown".to_string());

            return Ok(Some(Self { serial, alsa_card: Some(card) }));
        }

        // TODO: Try USB enumeration as fallback
        // This would use rusb or hidapi to detect the device
        // but requires proper udev rules for permissions.

        debug!("No Wave:3 device found");
        Ok(None)
    }

    /// Get the device serial number.
    #[must_use]
    pub fn serial(&self) -> &str {
        &self.serial
    }

    /// Get the ALSA card name if available.
    #[must_use]
    pub fn alsa_card(&self) -> Option<&str> {
        self.alsa_card.as_deref()
    }

    /// Find the ALSA card for Wave:3.
    fn find_alsa_card() -> HidResult<Option<String>> {
        // Read /proc/asound/cards to find Wave:3
        let cards =
            std::fs::read_to_string("/proc/asound/cards").map_err(|e| HidError::IoError(e))?;

        for line in cards.lines() {
            if line.contains("Wave:3") || line.contains("Wave 3") {
                // Extract card number from line like " 2 [Wave3          ]: USB-Audio - Wave:3"
                if let Some(num) = line.trim().split_whitespace().next() {
                    if let Ok(card_num) = num.parse::<u32>() {
                        return Ok(Some(format!("hw:{card_num}")));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get serial number from ALSA card info.
    fn get_serial_from_alsa(card: &str) -> Option<String> {
        // The serial might be in /proc/asound/cardX/usbid or similar
        // For now, return None and implement properly later
        None
    }
}

/// Device state for tracking changes.
#[derive(Debug, Clone, Default)]
pub struct DeviceState {
    /// Microphone mute state
    pub mic_muted: bool,
    /// Microphone gain (0.0 - 1.0)
    pub mic_gain: f32,
    /// Headphone volume (0.0 - 1.0)
    pub headphone_volume: f32,
}
