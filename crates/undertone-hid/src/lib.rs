//! Undertone HID - Wave:3 hardware integration.
//!
//! This crate provides hardware control for the Elgato Wave:3 microphone,
//! including mute sync, gain control, and LED state.
//!
//! **Note**: The Wave:3 uses a vendor-specific USB interface (not standard HID),
//! so this implementation may require USB protocol reverse engineering.
//! For now, we use ALSA as a fallback for basic mic control.

pub mod alsa_fallback;
pub mod device;
pub mod error;

pub use device::Wave3Device;
pub use error::{HidError, HidResult};
