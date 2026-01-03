//! Daemon configuration.

use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Daemon settings
    #[serde(default)]
    pub daemon: DaemonConfig,
    /// Database settings
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Channel settings
    #[serde(default)]
    pub channels: ChannelsConfig,
    /// Device settings
    #[serde(default)]
    pub device: DeviceConfig,
}

/// Daemon-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self { log_level: default_log_level() }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Database settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    /// Database path (optional, uses default if not set)
    pub path: Option<PathBuf>,
}

/// Channel settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelsConfig {
    /// Default channel names
    #[serde(default = "default_channels")]
    pub defaults: Vec<String>,
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self { defaults: default_channels() }
    }
}

fn default_channels() -> Vec<String> {
    vec![
        "system".to_string(),
        "voice".to_string(),
        "music".to_string(),
        "browser".to_string(),
        "game".to_string(),
    ]
}

/// Device settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// USB Vendor ID
    #[serde(default = "default_vid")]
    pub vendor_id: String,
    /// USB Product ID
    #[serde(default = "default_pid")]
    pub product_id: String,
    /// Enable HID control
    #[serde(default = "default_true")]
    pub hid_enabled: bool,
    /// Use ALSA fallback
    #[serde(default = "default_true")]
    pub alsa_fallback: bool,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            vendor_id: default_vid(),
            product_id: default_pid(),
            hid_enabled: true,
            alsa_fallback: true,
        }
    }
}

fn default_vid() -> String {
    "0fd9".to_string()
}

fn default_pid() -> String {
    "0070".to_string()
}

fn default_true() -> bool {
    true
}

/// Load configuration from file or defaults.
pub fn load_config() -> Result<Config> {
    let config_path = config_path()?;

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {config_path:?}"))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {config_path:?}"))?;
        Ok(config)
    } else {
        info!(?config_path, "Config file not found, using defaults");
        Ok(Config::default())
    }
}

/// Get the configuration file path.
fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "undertone", "Undertone")
        .context("Could not determine config directory")?;
    Ok(dirs.config_dir().join("config.toml"))
}
