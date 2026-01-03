//! Wave:3 device detection and control.

use tracing::{debug, info};

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
        // Try USB enumeration first
        if let Some(device) = Self::detect_usb()? {
            return Ok(Some(device));
        }

        // Fall back to ALSA card detection
        if let Some(card) = Self::find_alsa_card()? {
            info!(card = %card, "Wave:3 detected via ALSA");
            let serial = Self::get_serial_from_alsa(&card).unwrap_or_else(|| "unknown".to_string());
            return Ok(Some(Self { serial, alsa_card: Some(card) }));
        }

        debug!("No Wave:3 device found");
        Ok(None)
    }

    /// Detect Wave:3 via USB enumeration.
    fn detect_usb() -> HidResult<Option<Self>> {
        let devices = match rusb::devices() {
            Ok(d) => d,
            Err(e) => {
                debug!(error = %e, "Failed to enumerate USB devices");
                return Ok(None);
            }
        };

        for device in devices.iter() {
            let Ok(desc) = device.device_descriptor() else {
                continue;
            };

            if desc.vendor_id() == ELGATO_VID && desc.product_id() == WAVE3_PID {
                // Found Wave:3!
                let serial = Self::get_usb_serial(&device).unwrap_or_else(|| "unknown".to_string());
                let alsa_card = Self::find_alsa_card().ok().flatten();

                info!(
                    serial = %serial,
                    bus = device.bus_number(),
                    address = device.address(),
                    "Wave:3 detected via USB"
                );

                return Ok(Some(Self { serial, alsa_card }));
            }
        }

        Ok(None)
    }

    /// Get the serial number from a USB device.
    fn get_usb_serial<T: rusb::UsbContext>(device: &rusb::Device<T>) -> Option<String> {
        let desc = device.device_descriptor().ok()?;
        let handle = device.open().ok()?;

        // Try to get serial string
        if desc.serial_number_string_index().is_some() {
            handle.read_serial_number_string_ascii(&desc).ok()
        } else {
            None
        }
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
        let cards = std::fs::read_to_string("/proc/asound/cards").map_err(HidError::IoError)?;

        for line in cards.lines() {
            if line.contains("Wave:3") || line.contains("Wave 3") {
                // Extract card number from line like " 2 [Wave3          ]: USB-Audio - Wave:3"
                if let Some(num) = line.split_whitespace().next()
                    && let Ok(card_num) = num.parse::<u32>()
                {
                    return Ok(Some(format!("hw:{card_num}")));
                }
            }
        }

        Ok(None)
    }

    /// Get serial number from ALSA card info.
    fn get_serial_from_alsa(_card: &str) -> Option<String> {
        // The serial might be in /proc/asound/cardX/usbid or similar
        // For now, return None and use USB detection instead
        None
    }
}

/// Check if a Wave:3 device is currently connected.
#[must_use]
pub fn is_wave3_connected() -> bool {
    let Ok(devices) = rusb::devices() else {
        return false;
    };

    for device in devices.iter() {
        if let Ok(desc) = device.device_descriptor()
            && desc.vendor_id() == ELGATO_VID
            && desc.product_id() == WAVE3_PID
        {
            return true;
        }
    }

    false
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
