//! PipeWire link management.

use serde::{Deserialize, Serialize};

/// Information about a PipeWire link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    /// PipeWire object ID
    pub id: u32,
    /// Output (source) node ID
    pub output_node: u32,
    /// Output port ID
    pub output_port: u32,
    /// Input (destination) node ID
    pub input_node: u32,
    /// Input port ID
    pub input_port: u32,
    /// Link state
    pub state: LinkState,
    /// Whether this link is managed by Undertone
    pub is_undertone_managed: bool,
}

/// State of a PipeWire link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkState {
    /// Link is being set up
    Init,
    /// Link is being negotiated
    Negotiating,
    /// Link is allocating buffers
    Allocating,
    /// Link is paused
    Paused,
    /// Link is active
    Active,
    /// Link encountered an error
    Error,
    /// Link is unlinked
    Unlinked,
}

/// Parameters for creating a link.
#[derive(Debug, Clone)]
pub struct LinkParams {
    /// Output node ID
    pub output_node: u32,
    /// Output port ID
    pub output_port: u32,
    /// Input node ID
    pub input_node: u32,
    /// Input port ID
    pub input_port: u32,
    /// Whether the link should persist after nodes disconnect
    pub linger: bool,
}

impl LinkParams {
    /// Create link parameters.
    #[must_use]
    pub fn new(output_node: u32, output_port: u32, input_node: u32, input_port: u32) -> Self {
        Self { output_node, output_port, input_node, input_port, linger: true }
    }
}
