//! PipeWire node management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a PipeWire node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// PipeWire object ID
    pub id: u32,
    /// Node name
    pub name: String,
    /// Node description
    pub description: Option<String>,
    /// Media class (Audio/Sink, Audio/Source, etc.)
    pub media_class: Option<String>,
    /// Application name (for client nodes)
    pub application_name: Option<String>,
    /// Binary name (for client nodes)
    pub binary_name: Option<String>,
    /// Process ID
    pub pid: Option<u32>,
    /// Whether this node is managed by Undertone
    pub is_undertone_managed: bool,
    /// All node properties
    pub properties: HashMap<String, String>,
}

impl NodeInfo {
    /// Check if this is an audio sink.
    #[must_use]
    pub fn is_sink(&self) -> bool {
        self.media_class
            .as_ref()
            .map_or(false, |c| c.contains("Sink"))
    }

    /// Check if this is an audio source.
    #[must_use]
    pub fn is_source(&self) -> bool {
        self.media_class
            .as_ref()
            .map_or(false, |c| c.contains("Source"))
    }

    /// Check if this is a Wave:3 node.
    #[must_use]
    pub fn is_wave3(&self) -> bool {
        self.name.starts_with("wave3-")
            || self.name.contains("Elgato")
            || self.name.contains("Wave:3")
    }

    /// Check if this is an Undertone channel node.
    #[must_use]
    pub fn is_undertone_channel(&self) -> bool {
        self.name.starts_with("ut-ch-")
    }

    /// Check if this is an Undertone mix node.
    #[must_use]
    pub fn is_undertone_mix(&self) -> bool {
        self.name.starts_with("ut-stream-") || self.name.starts_with("ut-monitor-")
    }
}

/// Information about a PipeWire port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    /// PipeWire object ID
    pub id: u32,
    /// Port name
    pub name: String,
    /// Port direction (in/out)
    pub direction: PortDirection,
    /// Parent node ID
    pub node_id: u32,
    /// Audio channel position (FL, FR, etc.)
    pub channel: Option<String>,
}

/// Port direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    /// Input port (receives audio)
    Input,
    /// Output port (sends audio)
    Output,
}

/// Properties for creating a virtual sink node.
#[derive(Debug, Clone)]
pub struct VirtualSinkProps {
    /// Node name (e.g., "ut-ch-music")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Number of audio channels (default: 2 for stereo)
    pub channels: u32,
    /// Audio channel positions (e.g., "FL,FR")
    pub positions: String,
}

impl VirtualSinkProps {
    /// Create properties for a stereo virtual sink.
    #[must_use]
    pub fn stereo(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            channels: 2,
            positions: "FL,FR".to_string(),
        }
    }

    /// Create properties for a mono virtual sink.
    #[must_use]
    pub fn mono(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            channels: 1,
            positions: "MONO".to_string(),
        }
    }
}
