//! PipeWire graph management.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use crate::link::LinkInfo;
use crate::node::{NodeInfo, PortDirection, PortInfo};

/// Manages the PipeWire audio graph.
///
/// This is the main interface for interacting with PipeWire. It maintains
/// a cached view of the current graph state and provides methods for
/// creating/destroying nodes and links.
pub struct GraphManager {
    /// Cached nodes by ID
    nodes: Arc<RwLock<HashMap<u32, NodeInfo>>>,
    /// Cached ports by ID
    ports: Arc<RwLock<HashMap<u32, PortInfo>>>,
    /// Cached links by ID
    links: Arc<RwLock<HashMap<u32, LinkInfo>>>,
    /// Nodes created by Undertone (name -> id)
    created_nodes: Arc<RwLock<HashMap<String, u32>>>,
    /// Links created by Undertone (description -> id)
    created_links: Arc<RwLock<HashMap<String, u32>>>,
}

impl GraphManager {
    /// Create a new graph manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            ports: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(HashMap::new())),
            created_nodes: Arc::new(RwLock::new(HashMap::new())),
            created_links: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a node to the cache.
    pub fn add_node(&self, node: NodeInfo) {
        debug!(id = node.id, name = %node.name, "Node added to graph");
        self.nodes.write().insert(node.id, node);
    }

    /// Remove a node from the cache.
    pub fn remove_node(&self, id: u32) {
        if let Some(node) = self.nodes.write().remove(&id) {
            debug!(id, name = %node.name, "Node removed from graph");
        }
    }

    /// Get a node by ID.
    #[must_use]
    pub fn get_node(&self, id: u32) -> Option<NodeInfo> {
        self.nodes.read().get(&id).cloned()
    }

    /// Get a node by name.
    #[must_use]
    pub fn get_node_by_name(&self, name: &str) -> Option<NodeInfo> {
        self.nodes.read().values().find(|n| n.name == name).cloned()
    }

    /// Get all nodes.
    #[must_use]
    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.read().values().cloned().collect()
    }

    /// Add a port to the cache.
    pub fn add_port(&self, port: PortInfo) {
        debug!(id = port.id, name = %port.name, node_id = port.node_id, "Port added");
        self.ports.write().insert(port.id, port);
    }

    /// Remove a port from the cache.
    pub fn remove_port(&self, id: u32) {
        self.ports.write().remove(&id);
    }

    /// Get ports for a node.
    #[must_use]
    pub fn get_ports_for_node(&self, node_id: u32) -> Vec<PortInfo> {
        self.ports.read().values().filter(|p| p.node_id == node_id).cloned().collect()
    }

    /// Get a port by node ID and port name.
    #[must_use]
    pub fn get_port_by_name(&self, node_id: u32, port_name: &str) -> Option<PortInfo> {
        self.ports.read().values().find(|p| p.node_id == node_id && p.name == port_name).cloned()
    }

    /// Get input ports for a node.
    #[must_use]
    pub fn get_input_ports(&self, node_id: u32) -> Vec<PortInfo> {
        self.ports
            .read()
            .values()
            .filter(|p| p.node_id == node_id && p.direction == PortDirection::Input)
            .cloned()
            .collect()
    }

    /// Get output ports for a node (monitor ports for sinks).
    #[must_use]
    pub fn get_output_ports(&self, node_id: u32) -> Vec<PortInfo> {
        self.ports
            .read()
            .values()
            .filter(|p| p.node_id == node_id && p.direction == PortDirection::Output)
            .cloned()
            .collect()
    }

    /// Get a port by node ID and channel position (e.g., "FL", "FR").
    #[must_use]
    pub fn get_port_by_channel(
        &self,
        node_id: u32,
        direction: PortDirection,
        channel: &str,
    ) -> Option<PortInfo> {
        self.ports
            .read()
            .values()
            .find(|p| {
                p.node_id == node_id
                    && p.direction == direction
                    && p.channel.as_deref() == Some(channel)
            })
            .cloned()
    }

    /// Get all links.
    #[must_use]
    pub fn get_all_links(&self) -> Vec<LinkInfo> {
        self.links.read().values().cloned().collect()
    }

    /// Get a link by ID.
    #[must_use]
    pub fn get_link(&self, id: u32) -> Option<LinkInfo> {
        self.links.read().get(&id).cloned()
    }

    /// Get links involving a specific node (as source or destination).
    #[must_use]
    pub fn get_links_for_node(&self, node_id: u32) -> Vec<LinkInfo> {
        self.links
            .read()
            .values()
            .filter(|l| l.output_node == node_id || l.input_node == node_id)
            .cloned()
            .collect()
    }

    /// Check if a link exists between two nodes.
    #[must_use]
    pub fn has_link(&self, output_node: u32, input_node: u32) -> bool {
        self.links
            .read()
            .values()
            .any(|l| l.output_node == output_node && l.input_node == input_node)
    }

    /// Add a link to the cache.
    pub fn add_link(&self, link: LinkInfo) {
        debug!(id = link.id, "Link added to graph");
        self.links.write().insert(link.id, link);
    }

    /// Remove a link from the cache.
    pub fn remove_link(&self, id: u32) {
        self.links.write().remove(&id);
    }

    /// Get all Wave:3 nodes.
    #[must_use]
    pub fn get_wave3_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.read().values().filter(|n| n.is_wave3()).cloned().collect()
    }

    /// Find the Wave:3 headphone sink.
    ///
    /// First tries to find by the custom name "wave3-sink", then falls back
    /// to searching by device properties (vendor.id and product.id).
    #[must_use]
    pub fn find_wave3_sink(&self) -> Option<NodeInfo> {
        // First try the custom WirePlumber name
        if let Some(node) = self.get_node_by_name("wave3-sink") {
            return Some(node);
        }

        // Fall back to searching by device properties
        self.nodes
            .read()
            .values()
            .find(|n| {
                // Check if it's an Elgato Wave:3 sink
                let is_wave3_vendor =
                    n.properties.get("device.vendor.id").map_or(false, |v| v == "0x0fd9");
                let is_wave3_product =
                    n.properties.get("device.product.id").map_or(false, |v| v == "0x0070");
                let is_sink = n.media_class.as_ref().map_or(false, |c| c == "Audio/Sink");
                let is_alsa = n.name.starts_with("alsa_output");

                // Match by vendor/product ID or by name containing Elgato Wave
                let is_wave3_by_name =
                    n.name.contains("Elgato") && n.name.contains("Wave") && is_sink;
                let is_wave3_by_id = is_wave3_vendor && is_wave3_product && is_sink;

                (is_wave3_by_id || is_wave3_by_name || (is_alsa && n.is_wave3())) && is_sink
            })
            .cloned()
    }

    /// Find the Wave:3 microphone source.
    ///
    /// First tries to find by the custom name "wave3-source", then falls back
    /// to searching by device properties.
    #[must_use]
    pub fn find_wave3_source(&self) -> Option<NodeInfo> {
        // First try the custom WirePlumber name
        if let Some(node) = self.get_node_by_name("wave3-source") {
            return Some(node);
        }

        // Fall back to searching by device properties
        self.nodes
            .read()
            .values()
            .find(|n| {
                let is_source = n.media_class.as_ref().map_or(false, |c| c == "Audio/Source");
                let is_wave3_by_name =
                    (n.name.contains("Elgato") || n.name.contains("Wave")) && is_source;
                let is_alsa_input = n.name.starts_with("alsa_input");

                (is_wave3_by_name || (is_alsa_input && n.is_wave3())) && is_source
            })
            .cloned()
    }

    /// Get all Undertone channel nodes.
    #[must_use]
    pub fn get_undertone_channels(&self) -> Vec<NodeInfo> {
        self.nodes.read().values().filter(|n| n.is_undertone_channel()).cloned().collect()
    }

    /// Get all audio client nodes (apps producing audio).
    #[must_use]
    pub fn get_audio_clients(&self) -> Vec<NodeInfo> {
        self.nodes
            .read()
            .values()
            .filter(|n| {
                n.media_class.as_ref().map_or(false, |c| c == "Stream/Output/Audio")
                    && !n.is_undertone_managed
                    && !n.is_wave3()
            })
            .cloned()
            .collect()
    }

    /// Get all available audio output devices (sinks).
    ///
    /// Returns physical audio outputs like headphones, speakers, HDMI, etc.
    /// Excludes Undertone's virtual sinks.
    #[must_use]
    pub fn get_audio_output_devices(&self) -> Vec<NodeInfo> {
        self.nodes
            .read()
            .values()
            .filter(|n| {
                let is_sink = n.media_class.as_ref().map_or(false, |c| c == "Audio/Sink");
                // Exclude Undertone's virtual nodes
                let is_undertone = n.name.starts_with("ut-");
                // Exclude null sinks used internally
                let is_null = n.name.contains("null-sink");
                is_sink && !is_undertone && !is_null
            })
            .cloned()
            .collect()
    }

    /// Record that we created a node.
    pub fn record_created_node(&self, name: String, id: u32) {
        self.created_nodes.write().insert(name, id);
    }

    /// Record that we created a link.
    pub fn record_created_link(&self, description: String, id: u32) {
        self.created_links.write().insert(description, id);
    }

    /// Get the ID of a node we created by name.
    #[must_use]
    pub fn get_created_node_id(&self, name: &str) -> Option<u32> {
        self.created_nodes.read().get(name).copied()
    }

    /// Check if we have all expected Undertone nodes.
    #[must_use]
    pub fn has_all_undertone_nodes(&self, expected: &[&str]) -> bool {
        let created = self.created_nodes.read();
        expected.iter().all(|name| created.contains_key(*name))
    }

    /// Get all created nodes as a HashMap.
    #[must_use]
    pub fn get_created_nodes(&self) -> std::collections::HashMap<String, u32> {
        self.created_nodes.read().clone()
    }

    /// Get all created links as a HashMap.
    #[must_use]
    pub fn get_created_links(&self) -> std::collections::HashMap<String, u32> {
        self.created_links.read().clone()
    }
}

impl Default for GraphManager {
    fn default() -> Self {
        Self::new()
    }
}
