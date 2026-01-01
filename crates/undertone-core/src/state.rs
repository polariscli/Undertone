//! Daemon state machine and event handling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::channel::ChannelState;
use crate::mixer::MixerState;
use crate::routing::AppRoute;

/// Current state of the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonState {
    /// Initial state, loading configuration
    Initializing,
    /// Waiting for Wave:3 device to appear
    WaitingForDevice,
    /// Creating PipeWire nodes
    CreatingNodes,
    /// Normal operation
    Running,
    /// Device disconnected, nodes preserved
    DeviceDisconnected,
    /// PipeWire/WirePlumber restarted, reconciling
    Reconciling,
    /// Graceful shutdown in progress
    ShuttingDown,
    /// Fatal error state
    Error(String),
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::Initializing
    }
}

/// Events that can trigger state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum DaemonEvent {
    // Initialization events
    ConfigLoaded,
    DatabaseReady,
    PipeWireConnected,

    // Device events
    Wave3Detected { serial: String },
    Wave3Disconnected,

    // PipeWire events
    NodeCreated { id: u32, name: String },
    NodeRemoved { id: u32 },
    LinkCreated { id: u32 },
    LinkRemoved { id: u32 },
    ClientAppeared { id: u32, name: String, pid: u32 },
    ClientDisappeared { id: u32 },

    // External events
    PipeWireRestarted,
    WirePlumberRestarted,

    // Control events
    ShutdownRequested,
    ReconcileRequested,
}

/// Complete snapshot of the daemon's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Current daemon state
    pub state: DaemonState,
    /// Whether Wave:3 is connected
    pub device_connected: bool,
    /// Wave:3 serial number (if connected)
    pub device_serial: Option<String>,
    /// Channel states
    pub channels: Vec<ChannelState>,
    /// Active app routes
    pub app_routes: Vec<AppRoute>,
    /// Mixer state
    pub mixer: MixerState,
    /// Active profile name
    pub active_profile: String,
    /// PipeWire node IDs we've created
    pub created_nodes: HashMap<String, u32>,
    /// PipeWire link IDs we've created
    pub created_links: HashMap<String, u32>,
}

impl Default for StateSnapshot {
    fn default() -> Self {
        Self {
            state: DaemonState::Initializing,
            device_connected: false,
            device_serial: None,
            channels: Vec::new(),
            app_routes: Vec::new(),
            mixer: MixerState::default(),
            active_profile: "Default".to_string(),
            created_nodes: HashMap::new(),
            created_links: HashMap::new(),
        }
    }
}
