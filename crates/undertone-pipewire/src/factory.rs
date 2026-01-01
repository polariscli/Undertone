//! PipeWire node factory for creating virtual sinks and sources.
//!
//! This module provides functions to create virtual audio nodes that Undertone
//! uses for channel mixing and routing.

use std::sync::mpsc as std_mpsc;

use pipewire::core::Core;
use pipewire::properties::properties;
use pipewire::proxy::ProxyT;
use tracing::{debug, info};

use crate::error::{PwError, PwResult};
use crate::node::VirtualSinkProps;

/// SPA property keys for volume control
pub mod spa_props {
    /// Volume control property (0-1.0 linear scale)
    pub const SPA_PROP_VOLUME: u32 = 65539;
    /// Mute control property
    pub const SPA_PROP_MUTE: u32 = 65540;
    /// Per-channel volumes array
    pub const SPA_PROP_CHANNEL_VOLUMES: u32 = 65544;
    /// Soft mute (software-based muting)
    pub const SPA_PROP_SOFT_MUTE: u32 = 65551;
    /// Soft volumes array (software-based volume)
    pub const SPA_PROP_SOFT_VOLUMES: u32 = 65552;
}

/// Result of node creation
#[derive(Debug, Clone)]
pub struct CreatedNode {
    pub id: u32,
    pub name: String,
}

/// Request to create a node
#[derive(Debug, Clone)]
pub enum FactoryRequest {
    /// Create a virtual sink
    CreateSink(VirtualSinkProps),
    /// Create a volume filter node (a sink that passes audio through with volume control)
    CreateVolumeFilter {
        /// Node name (e.g., "ut-ch-music-stream-vol")
        name: String,
        /// Human-readable description
        description: String,
        /// Number of channels (2 for stereo)
        channels: u32,
    },
    /// Create a link between ports
    CreateLink { output_node: u32, output_port: String, input_node: u32, input_port: String },
    /// Set volume on a node
    SetNodeVolume {
        /// Node ID to set volume on
        node_id: u32,
        /// Volume level (0.0 - 1.0)
        volume: f32,
    },
    /// Set mute state on a node
    SetNodeMute {
        /// Node ID to set mute on
        node_id: u32,
        /// Mute state
        muted: bool,
    },
    /// Destroy a node by ID
    DestroyNode(u32),
    /// Destroy a link by ID
    DestroyLink(u32),
    /// Shutdown the factory
    Shutdown,
}

/// Response from factory operations
#[derive(Debug, Clone)]
pub enum FactoryResponse {
    /// Node was created
    NodeCreated(CreatedNode),
    /// Link was created
    LinkCreated { id: u32 },
    /// Volume was set on a node
    VolumeSet { node_id: u32 },
    /// Mute was set on a node
    MuteSet { node_id: u32 },
    /// Node was destroyed
    NodeDestroyed { id: u32 },
    /// Link was destroyed
    LinkDestroyed { id: u32 },
    /// Operation failed
    Error(String),
}

/// Factory thread for creating PipeWire objects.
///
/// This runs in the same thread as the monitor since PipeWire objects
/// must be created on the same thread as the main loop.
pub struct NodeFactory {
    request_rx: std_mpsc::Receiver<FactoryRequest>,
    response_tx: std_mpsc::Sender<FactoryResponse>,
}

impl NodeFactory {
    /// Create a new node factory.
    pub fn new(
        request_rx: std_mpsc::Receiver<FactoryRequest>,
        response_tx: std_mpsc::Sender<FactoryResponse>,
    ) -> Self {
        Self { request_rx, response_tx }
    }

    /// Process pending requests (called from PipeWire main loop)
    ///
    /// Note: This method is used by the legacy NodeFactory pattern.
    /// The PipeWireRuntime uses its own handlers in runtime.rs.
    pub fn process_requests(&self, core: &Core) {
        while let Ok(request) = self.request_rx.try_recv() {
            match request {
                FactoryRequest::CreateSink(props) => match self.create_sink(core, &props) {
                    Ok(node) => {
                        let _ = self.response_tx.send(FactoryResponse::NodeCreated(node));
                    }
                    Err(e) => {
                        let _ = self.response_tx.send(FactoryResponse::Error(e.to_string()));
                    }
                },
                FactoryRequest::CreateVolumeFilter { name, description, channels } => {
                    // Volume filters are handled the same as sinks in this context
                    let props = VirtualSinkProps {
                        name,
                        description,
                        channels,
                        positions: if channels == 1 {
                            "MONO".to_string()
                        } else {
                            "FL,FR".to_string()
                        },
                    };
                    match self.create_sink(core, &props) {
                        Ok(node) => {
                            let _ = self.response_tx.send(FactoryResponse::NodeCreated(node));
                        }
                        Err(e) => {
                            let _ = self.response_tx.send(FactoryResponse::Error(e.to_string()));
                        }
                    }
                }
                FactoryRequest::CreateLink { output_node, output_port, input_node, input_port } => {
                    match self.create_link(core, output_node, &output_port, input_node, &input_port)
                    {
                        Ok(id) => {
                            let _ = self.response_tx.send(FactoryResponse::LinkCreated { id });
                        }
                        Err(e) => {
                            let _ = self.response_tx.send(FactoryResponse::Error(e.to_string()));
                        }
                    }
                }
                FactoryRequest::SetNodeVolume { node_id, .. } => {
                    // Volume control requires node proxy access, not implemented in this legacy path
                    let _ = self.response_tx.send(FactoryResponse::VolumeSet { node_id });
                }
                FactoryRequest::SetNodeMute { node_id, .. } => {
                    // Mute control requires node proxy access, not implemented in this legacy path
                    let _ = self.response_tx.send(FactoryResponse::MuteSet { node_id });
                }
                FactoryRequest::DestroyNode(id) => {
                    // Node destruction is handled by proxy.destroy()
                    // For now, we just acknowledge
                    let _ = self.response_tx.send(FactoryResponse::NodeDestroyed { id });
                }
                FactoryRequest::DestroyLink(id) => {
                    let _ = self.response_tx.send(FactoryResponse::LinkDestroyed { id });
                }
                FactoryRequest::Shutdown => {
                    info!("Factory received shutdown request");
                    break;
                }
            }
        }
    }

    fn create_sink(&self, core: &Core, props: &VirtualSinkProps) -> PwResult<CreatedNode> {
        info!(name = %props.name, "Creating virtual sink");

        // Create properties for the null-audio-sink
        let node_props = properties! {
            "factory.name" => "support.null-audio-sink",
            "node.name" => props.name.as_str(),
            "node.description" => props.description.as_str(),
            "media.class" => "Audio/Sink",
            "audio.channels" => props.channels.to_string().as_str(),
            "audio.position" => props.positions.as_str(),
            "undertone.managed" => "true",
            "node.passive" => "true",
            // Prevent auto-suspend
            "session.suspend-timeout-seconds" => "0",
        };

        // Create the node via the core
        // Note: This uses the adapter factory to create a null sink
        let proxy = core
            .create_object::<pipewire::node::Node>("adapter", &node_props)
            .map_err(|e| PwError::NodeCreationFailed(format!("Failed to create node: {e}")))?;

        let id = proxy.upcast_ref().id();

        debug!(id, name = %props.name, "Virtual sink created");

        Ok(CreatedNode { id, name: props.name.clone() })
    }

    fn create_link(
        &self,
        core: &Core,
        output_node: u32,
        output_port: &str,
        input_node: u32,
        input_port: &str,
    ) -> PwResult<u32> {
        info!(output_node, input_node, "Creating link");

        let link_props = properties! {
            "link.output.node" => output_node.to_string().as_str(),
            "link.output.port" => output_port,
            "link.input.node" => input_node.to_string().as_str(),
            "link.input.port" => input_port,
            "object.linger" => "true",
        };

        let proxy = core
            .create_object::<pipewire::link::Link>("link-factory", &link_props)
            .map_err(|e| PwError::LinkCreationFailed(format!("Failed to create link: {e}")))?;

        let id = proxy.upcast_ref().id();

        debug!(id, "Link created");

        Ok(id)
    }
}

/// Create a channel pair for factory communication
pub fn create_factory_channel() -> (
    std_mpsc::Sender<FactoryRequest>,
    std_mpsc::Receiver<FactoryResponse>,
    std_mpsc::Receiver<FactoryRequest>,
    std_mpsc::Sender<FactoryResponse>,
) {
    let (req_tx, req_rx) = std_mpsc::channel();
    let (resp_tx, resp_rx) = std_mpsc::channel();
    (req_tx, resp_rx, req_rx, resp_tx)
}
