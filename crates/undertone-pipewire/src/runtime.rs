//! PipeWire runtime that combines monitoring and node creation.
//!
//! This module provides a unified runtime that handles both graph monitoring
//! and node creation in a single PipeWire thread.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use libspa::param::ParamType;
use libspa::pod::serialize::PodSerializer;
use libspa::pod::{Object, Property, Value};
use libspa::utils::SpaTypes;
use pipewire::context::ContextRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::properties::properties;
use pipewire::proxy::ProxyT;
use pipewire::registry::GlobalObject;
use pipewire::spa::pod::Pod;
use pipewire::spa::utils::dict::DictRef;
use pipewire::types::ObjectType;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{PwError, PwResult};
use crate::factory::{CreatedNode, FactoryRequest, FactoryResponse, spa_props};
use crate::graph::GraphManager;
use crate::monitor::GraphEvent;
use crate::node::{NodeInfo, PortDirection, PortInfo, VirtualSinkProps};

/// PipeWire runtime handle for the async world.
pub struct PipeWireRuntime {
    /// Channel to send factory requests
    factory_tx: pipewire::channel::Sender<FactoryRequest>,
    /// Channel to receive factory responses
    factory_rx: std_mpsc::Receiver<FactoryResponse>,
    /// Graph manager (shared with monitor thread)
    graph: Arc<GraphManager>,
}

impl PipeWireRuntime {
    /// Spawn the PipeWire runtime and return a handle.
    pub fn spawn(graph: Arc<GraphManager>) -> (Self, mpsc::Receiver<GraphEvent>) {
        let (event_tx, event_rx) = mpsc::channel(256);
        let (factory_tx, factory_internal_rx) = pipewire::channel::channel();
        let (factory_internal_tx, factory_rx) = std_mpsc::channel();

        let graph_clone = Arc::clone(&graph);

        std::thread::Builder::new()
            .name("pipewire-runtime".to_string())
            .spawn(move || {
                if let Err(e) = run_pipewire_thread(
                    graph_clone,
                    event_tx,
                    factory_internal_rx,
                    factory_internal_tx,
                ) {
                    error!(error = %e, "PipeWire runtime failed");
                }
            })
            .expect("Failed to spawn PipeWire runtime thread");

        (Self { factory_tx, factory_rx, graph }, event_rx)
    }

    /// Create a virtual sink node.
    pub fn create_sink(&self, props: VirtualSinkProps) -> PwResult<CreatedNode> {
        self.factory_tx
            .send(FactoryRequest::CreateSink(props))
            .map_err(|_| PwError::MainLoopError("Factory channel closed".to_string()))?;

        // Wait for response with timeout
        match self.factory_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(FactoryResponse::NodeCreated(node)) => Ok(node),
            Ok(FactoryResponse::Error(e)) => Err(PwError::NodeCreationFailed(e)),
            Ok(_) => Err(PwError::NodeCreationFailed("Unexpected response".to_string())),
            Err(_) => Err(PwError::NodeCreationFailed("Timeout waiting for response".to_string())),
        }
    }

    /// Create all channel sinks.
    pub fn create_channel_sinks(&self, channels: &[&str]) -> PwResult<Vec<CreatedNode>> {
        let mut nodes = Vec::new();

        for name in channels {
            let props = VirtualSinkProps::stereo(
                &format!("ut-ch-{name}"),
                &format!("Undertone: {} Channel", capitalize(name)),
            );

            match self.create_sink(props) {
                Ok(node) => {
                    info!(name = %node.name, id = node.id, "Created channel sink");
                    nodes.push(node);
                }
                Err(e) => {
                    error!(channel = %name, error = %e, "Failed to create channel sink");
                    return Err(e);
                }
            }
        }

        Ok(nodes)
    }

    /// Create mix nodes (stream-mix, monitor-mix).
    pub fn create_mix_nodes(&self) -> PwResult<Vec<CreatedNode>> {
        let mut nodes = Vec::new();

        let mix_configs = [
            ("ut-stream-mix", "Undertone: Stream Mix"),
            ("ut-monitor-mix", "Undertone: Monitor Mix"),
        ];

        for (name, desc) in mix_configs {
            let props = VirtualSinkProps::stereo(name, desc);
            match self.create_sink(props) {
                Ok(node) => {
                    info!(name = %node.name, id = node.id, "Created mix node");
                    nodes.push(node);
                }
                Err(e) => {
                    error!(node = %name, error = %e, "Failed to create mix node");
                    return Err(e);
                }
            }
        }

        Ok(nodes)
    }

    /// Get the graph manager.
    pub fn graph(&self) -> &Arc<GraphManager> {
        &self.graph
    }

    /// Create a link between two nodes.
    ///
    /// Links connect output ports from the source node to input ports on the
    /// destination node. For audio routing, this typically connects monitor
    /// ports of a sink to input ports of another sink.
    ///
    /// # Arguments
    /// * `output_node` - The source node ID
    /// * `output_port` - The output port name (e.g., "monitor_FL")
    /// * `input_node` - The destination node ID
    /// * `input_port` - The input port name (e.g., "input_FL")
    pub fn create_link(
        &self,
        output_node: u32,
        output_port: &str,
        input_node: u32,
        input_port: &str,
    ) -> PwResult<u32> {
        self.factory_tx
            .send(FactoryRequest::CreateLink {
                output_node,
                output_port: output_port.to_string(),
                input_node,
                input_port: input_port.to_string(),
            })
            .map_err(|_| PwError::MainLoopError("Factory channel closed".to_string()))?;

        match self.factory_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(FactoryResponse::LinkCreated { id }) => Ok(id),
            Ok(FactoryResponse::Error(e)) => Err(PwError::LinkCreationFailed(e)),
            Ok(_) => Err(PwError::LinkCreationFailed("Unexpected response".to_string())),
            Err(_) => Err(PwError::LinkCreationFailed("Timeout waiting for response".to_string())),
        }
    }

    /// Create stereo links between two nodes.
    ///
    /// This creates two links: one for the left channel (FL) and one for
    /// the right channel (FR). This is the common case for stereo audio routing.
    ///
    /// For sink-to-sink routing, this links monitor ports (output) to
    /// playback ports (input). PipeWire uses:
    /// - "monitor_FL"/"monitor_FR" for monitor output ports
    /// - "playback_FL"/"playback_FR" for playback input ports
    pub fn create_stereo_links(&self, output_node: u32, input_node: u32) -> PwResult<(u32, u32)> {
        let left_id = self.create_link(output_node, "monitor_FL", input_node, "playback_FL")?;
        let right_id = self.create_link(output_node, "monitor_FR", input_node, "playback_FR")?;
        Ok((left_id, right_id))
    }

    /// Destroy a link by ID.
    pub fn destroy_link(&self, id: u32) -> PwResult<()> {
        self.factory_tx
            .send(FactoryRequest::DestroyLink(id))
            .map_err(|_| PwError::MainLoopError("Factory channel closed".to_string()))?;

        match self.factory_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(FactoryResponse::LinkDestroyed { id: _ }) => Ok(()),
            Ok(FactoryResponse::Error(e)) => Err(PwError::LinkCreationFailed(e)),
            Ok(_) => Err(PwError::LinkCreationFailed("Unexpected response".to_string())),
            Err(_) => Err(PwError::LinkCreationFailed("Timeout waiting for response".to_string())),
        }
    }

    /// Create links from all channel sinks to mix nodes through volume filter nodes.
    ///
    /// This creates the full audio routing topology:
    /// - channel → stream-vol-filter → stream-mix
    /// - channel → monitor-vol-filter → monitor-mix
    ///
    /// This enables independent volume control for stream and monitor paths per channel.
    ///
    /// Returns a vector of (link_description, link_id) tuples for tracking.
    pub fn create_channel_to_mix_links_with_filters(&self) -> PwResult<Vec<(String, u32)>> {
        let mut created_links = Vec::new();

        // Get mix node IDs
        let stream_mix_id = self
            .graph
            .get_created_node_id("ut-stream-mix")
            .ok_or_else(|| PwError::NodeNotFound("ut-stream-mix".to_string()))?;

        let monitor_mix_id = self
            .graph
            .get_created_node_id("ut-monitor-mix")
            .ok_or_else(|| PwError::NodeNotFound("ut-monitor-mix".to_string()))?;

        // Get all channel nodes (ut-ch-{name})
        let channel_nodes = self.graph.get_undertone_channels();

        for channel in &channel_nodes {
            let channel_id = channel.id;
            let channel_name = &channel.name;

            // Extract the base channel name (e.g., "music" from "ut-ch-music")
            let base_name = channel_name.strip_prefix("ut-ch-").unwrap_or(channel_name);

            // Get the stream volume filter node ID
            let stream_vol_name = format!("ut-ch-{base_name}-stream-vol");
            let stream_vol_id = self
                .graph
                .get_created_node_id(&stream_vol_name)
                .ok_or_else(|| PwError::NodeNotFound(stream_vol_name.clone()))?;

            // Get the monitor volume filter node ID
            let monitor_vol_name = format!("ut-ch-{base_name}-monitor-vol");
            let monitor_vol_id = self
                .graph
                .get_created_node_id(&monitor_vol_name)
                .ok_or_else(|| PwError::NodeNotFound(monitor_vol_name.clone()))?;

            // Link channel -> stream-vol-filter (stereo)
            match self.create_stereo_links(channel_id, stream_vol_id) {
                Ok((left_id, right_id)) => {
                    info!(
                        channel = %channel_name,
                        filter = %stream_vol_name,
                        "Linked channel to stream volume filter"
                    );
                    created_links.push((format!("{channel_name}->{stream_vol_name}:FL"), left_id));
                    created_links.push((format!("{channel_name}->{stream_vol_name}:FR"), right_id));
                }
                Err(e) => {
                    error!(
                        channel = %channel_name,
                        error = %e,
                        "Failed to link channel to stream volume filter"
                    );
                }
            }

            // Link stream-vol-filter -> stream-mix (stereo)
            match self.create_stereo_links(stream_vol_id, stream_mix_id) {
                Ok((left_id, right_id)) => {
                    info!(
                        filter = %stream_vol_name,
                        "Linked stream volume filter to stream-mix"
                    );
                    created_links.push((format!("{stream_vol_name}->stream-mix:FL"), left_id));
                    created_links.push((format!("{stream_vol_name}->stream-mix:FR"), right_id));
                }
                Err(e) => {
                    error!(
                        filter = %stream_vol_name,
                        error = %e,
                        "Failed to link stream volume filter to stream-mix"
                    );
                }
            }

            // Link channel -> monitor-vol-filter (stereo)
            match self.create_stereo_links(channel_id, monitor_vol_id) {
                Ok((left_id, right_id)) => {
                    info!(
                        channel = %channel_name,
                        filter = %monitor_vol_name,
                        "Linked channel to monitor volume filter"
                    );
                    created_links.push((format!("{channel_name}->{monitor_vol_name}:FL"), left_id));
                    created_links
                        .push((format!("{channel_name}->{monitor_vol_name}:FR"), right_id));
                }
                Err(e) => {
                    error!(
                        channel = %channel_name,
                        error = %e,
                        "Failed to link channel to monitor volume filter"
                    );
                }
            }

            // Link monitor-vol-filter -> monitor-mix (stereo)
            match self.create_stereo_links(monitor_vol_id, monitor_mix_id) {
                Ok((left_id, right_id)) => {
                    info!(
                        filter = %monitor_vol_name,
                        "Linked monitor volume filter to monitor-mix"
                    );
                    created_links.push((format!("{monitor_vol_name}->monitor-mix:FL"), left_id));
                    created_links.push((format!("{monitor_vol_name}->monitor-mix:FR"), right_id));
                }
                Err(e) => {
                    error!(
                        filter = %monitor_vol_name,
                        error = %e,
                        "Failed to link monitor volume filter to monitor-mix"
                    );
                }
            }
        }

        Ok(created_links)
    }

    /// Create links from all channel sinks directly to mix nodes (legacy, without volume control).
    ///
    /// This links the monitor ports of each channel sink to the input ports
    /// of both the stream-mix and monitor-mix nodes, enabling audio to flow
    /// from channels to both output paths.
    ///
    /// Note: This bypasses volume filter nodes and doesn't support per-channel volume.
    /// Use `create_channel_to_mix_links_with_filters` for volume control support.
    ///
    /// Returns a vector of (link_description, link_id) tuples for tracking.
    pub fn create_channel_to_mix_links(&self) -> PwResult<Vec<(String, u32)>> {
        let mut created_links = Vec::new();

        // Get mix node IDs
        let stream_mix_id = self
            .graph
            .get_created_node_id("ut-stream-mix")
            .ok_or_else(|| PwError::NodeNotFound("ut-stream-mix".to_string()))?;

        let monitor_mix_id = self
            .graph
            .get_created_node_id("ut-monitor-mix")
            .ok_or_else(|| PwError::NodeNotFound("ut-monitor-mix".to_string()))?;

        // Get all channel nodes
        let channel_nodes = self.graph.get_undertone_channels();

        for channel in &channel_nodes {
            let channel_id = channel.id;
            let channel_name = &channel.name;

            // Link channel -> stream-mix (stereo)
            match self.create_stereo_links(channel_id, stream_mix_id) {
                Ok((left_id, right_id)) => {
                    info!(
                        channel = %channel_name,
                        stream_mix_id,
                        "Linked channel to stream-mix"
                    );
                    created_links.push((format!("{channel_name}->stream-mix:FL"), left_id));
                    created_links.push((format!("{channel_name}->stream-mix:FR"), right_id));
                }
                Err(e) => {
                    error!(
                        channel = %channel_name,
                        error = %e,
                        "Failed to link channel to stream-mix"
                    );
                }
            }

            // Link channel -> monitor-mix (stereo)
            match self.create_stereo_links(channel_id, monitor_mix_id) {
                Ok((left_id, right_id)) => {
                    info!(
                        channel = %channel_name,
                        monitor_mix_id,
                        "Linked channel to monitor-mix"
                    );
                    created_links.push((format!("{channel_name}->monitor-mix:FL"), left_id));
                    created_links.push((format!("{channel_name}->monitor-mix:FR"), right_id));
                }
                Err(e) => {
                    error!(
                        channel = %channel_name,
                        error = %e,
                        "Failed to link channel to monitor-mix"
                    );
                }
            }
        }

        Ok(created_links)
    }

    /// Link the monitor-mix output to the Wave:3 headphone sink.
    ///
    /// This enables local monitoring through the headphones.
    pub fn link_monitor_to_headphones(&self) -> PwResult<(u32, u32)> {
        let monitor_mix_id = self
            .graph
            .get_created_node_id("ut-monitor-mix")
            .ok_or_else(|| PwError::NodeNotFound("ut-monitor-mix".to_string()))?;

        // Find the Wave:3 sink node
        let wave3_sink = self
            .graph
            .get_node_by_name("wave3-sink")
            .ok_or_else(|| PwError::NodeNotFound("wave3-sink".to_string()))?;

        info!(
            monitor_mix_id,
            wave3_sink_id = wave3_sink.id,
            "Linking monitor-mix to Wave:3 headphones"
        );

        self.create_stereo_links(monitor_mix_id, wave3_sink.id)
    }

    /// Create a volume filter node.
    ///
    /// This creates a null-audio-sink that can be used as a volume control point
    /// in the audio routing graph. The node supports volume and mute control
    /// via `set_node_volume` and `set_node_mute`.
    pub fn create_volume_filter(
        &self,
        name: &str,
        description: &str,
        channels: u32,
    ) -> PwResult<CreatedNode> {
        self.factory_tx
            .send(FactoryRequest::CreateVolumeFilter {
                name: name.to_string(),
                description: description.to_string(),
                channels,
            })
            .map_err(|_| PwError::MainLoopError("Factory channel closed".to_string()))?;

        match self.factory_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(FactoryResponse::NodeCreated(node)) => Ok(node),
            Ok(FactoryResponse::Error(e)) => Err(PwError::NodeCreationFailed(e)),
            Ok(_) => Err(PwError::NodeCreationFailed("Unexpected response".to_string())),
            Err(_) => Err(PwError::NodeCreationFailed("Timeout waiting for response".to_string())),
        }
    }

    /// Create volume filter nodes for all channels.
    ///
    /// For each channel, this creates two filter nodes:
    /// - `ut-ch-{name}-stream-vol` for stream mix volume control
    /// - `ut-ch-{name}-monitor-vol` for monitor mix volume control
    ///
    /// Returns a vector of (filter_name, node_id) pairs.
    pub fn create_channel_volume_filters(&self, channels: &[&str]) -> PwResult<Vec<(String, u32)>> {
        let mut filters = Vec::new();

        for channel in channels {
            // Create stream volume filter
            let stream_name = format!("ut-ch-{channel}-stream-vol");
            let stream_desc = format!("Undertone: {} Stream Volume", capitalize(channel));
            match self.create_volume_filter(&stream_name, &stream_desc, 2) {
                Ok(node) => {
                    info!(name = %node.name, id = node.id, "Created stream volume filter");
                    filters.push((node.name, node.id));
                }
                Err(e) => {
                    error!(channel = %channel, error = %e, "Failed to create stream volume filter");
                    return Err(e);
                }
            }

            // Create monitor volume filter
            let monitor_name = format!("ut-ch-{channel}-monitor-vol");
            let monitor_desc = format!("Undertone: {} Monitor Volume", capitalize(channel));
            match self.create_volume_filter(&monitor_name, &monitor_desc, 2) {
                Ok(node) => {
                    info!(name = %node.name, id = node.id, "Created monitor volume filter");
                    filters.push((node.name, node.id));
                }
                Err(e) => {
                    error!(channel = %channel, error = %e, "Failed to create monitor volume filter");
                    return Err(e);
                }
            }
        }

        Ok(filters)
    }

    /// Set volume on a node.
    ///
    /// # Arguments
    /// * `node_id` - The PipeWire node ID
    /// * `volume` - Volume level from 0.0 (silent) to 1.0 (full volume)
    pub fn set_node_volume(&self, node_id: u32, volume: f32) -> PwResult<()> {
        let volume = volume.clamp(0.0, 1.0);

        self.factory_tx
            .send(FactoryRequest::SetNodeVolume { node_id, volume })
            .map_err(|_| PwError::MainLoopError("Factory channel closed".to_string()))?;

        match self.factory_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(FactoryResponse::VolumeSet { .. }) => Ok(()),
            Ok(FactoryResponse::Error(e)) => Err(PwError::VolumeControlFailed(e)),
            Ok(_) => Err(PwError::VolumeControlFailed("Unexpected response".to_string())),
            Err(_) => Err(PwError::VolumeControlFailed("Timeout waiting for response".to_string())),
        }
    }

    /// Set mute state on a node.
    ///
    /// # Arguments
    /// * `node_id` - The PipeWire node ID
    /// * `muted` - True to mute, false to unmute
    pub fn set_node_mute(&self, node_id: u32, muted: bool) -> PwResult<()> {
        self.factory_tx
            .send(FactoryRequest::SetNodeMute { node_id, muted })
            .map_err(|_| PwError::MainLoopError("Factory channel closed".to_string()))?;

        match self.factory_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(FactoryResponse::MuteSet { .. }) => Ok(()),
            Ok(FactoryResponse::Error(e)) => Err(PwError::VolumeControlFailed(e)),
            Ok(_) => Err(PwError::VolumeControlFailed("Unexpected response".to_string())),
            Err(_) => Err(PwError::VolumeControlFailed("Timeout waiting for response".to_string())),
        }
    }

    /// Request shutdown of the PipeWire thread.
    pub fn shutdown(&self) {
        let _ = self.factory_tx.send(FactoryRequest::Shutdown);
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Run the PipeWire thread - this combines monitoring and node creation.
fn run_pipewire_thread(
    graph: Arc<GraphManager>,
    event_tx: mpsc::Sender<GraphEvent>,
    factory_rx: pipewire::channel::Receiver<FactoryRequest>,
    factory_tx: std_mpsc::Sender<FactoryResponse>,
) -> PwResult<()> {
    // Initialize PipeWire
    pipewire::init();

    info!("PipeWire runtime starting...");

    // Create the main loop
    let main_loop = MainLoopRc::new(None)
        .map_err(|e| PwError::MainLoopError(format!("Failed to create main loop: {e}")))?;

    let context = ContextRc::new(&main_loop, None)
        .map_err(|e| PwError::ConnectionFailed(format!("Failed to create context: {e}")))?;

    let core = context
        .connect_rc(None)
        .map_err(|e| PwError::ConnectionFailed(format!("Failed to connect: {e}")))?;

    let registry = core
        .get_registry_rc()
        .map_err(|e| PwError::RegistryError(format!("Failed to get registry: {e}")))?;

    info!("Connected to PipeWire");
    let _ = event_tx.blocking_send(GraphEvent::Connected);

    // Track nodes for removal events
    let nodes: Rc<RefCell<HashMap<u32, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let nodes_remove = Rc::clone(&nodes);

    // Clone for closures
    let event_tx_global = event_tx.clone();
    let event_tx_remove = event_tx.clone();
    let graph_global = Arc::clone(&graph);

    // Set up registry listener
    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            handle_global(&event_tx_global, &graph_global, &nodes, global);
        })
        .global_remove(move |id| {
            handle_global_remove(&event_tx_remove, &nodes_remove, id);
        })
        .register();

    // Clone main loop and core for factory callback
    let main_loop_for_shutdown = main_loop.clone();
    let core_for_factory = core.clone();

    // Storage for created node proxies - must be kept alive to prevent node destruction
    // Use HashMap to enable looking up nodes by ID for volume control
    let node_proxies: Rc<RefCell<HashMap<u32, pipewire::node::Node>>> =
        Rc::new(RefCell::new(HashMap::new()));
    let link_proxies: Rc<RefCell<Vec<pipewire::link::Link>>> = Rc::new(RefCell::new(Vec::new()));
    let node_proxies_clone = Rc::clone(&node_proxies);
    let node_proxies_volume = Rc::clone(&node_proxies);
    let node_proxies_mute = Rc::clone(&node_proxies);
    let link_proxies_clone = Rc::clone(&link_proxies);

    // Attach factory request receiver to the loop
    let _factory_receiver = factory_rx.attach(main_loop.loop_(), move |request| match request {
        FactoryRequest::CreateSink(props) => {
            match create_virtual_sink(&core_for_factory, &props, &node_proxies_clone) {
                Ok(node) => {
                    let _ = factory_tx.send(FactoryResponse::NodeCreated(node));
                }
                Err(e) => {
                    let _ = factory_tx.send(FactoryResponse::Error(e.to_string()));
                }
            }
        }
        FactoryRequest::CreateVolumeFilter { name, description, channels } => {
            match create_volume_filter(
                &core_for_factory,
                &name,
                &description,
                channels,
                &node_proxies_clone,
            ) {
                Ok(node) => {
                    let _ = factory_tx.send(FactoryResponse::NodeCreated(node));
                }
                Err(e) => {
                    let _ = factory_tx.send(FactoryResponse::Error(e.to_string()));
                }
            }
        }
        FactoryRequest::CreateLink { output_node, output_port, input_node, input_port } => {
            match create_link(
                &core_for_factory,
                output_node,
                &output_port,
                input_node,
                &input_port,
                &link_proxies_clone,
            ) {
                Ok(id) => {
                    let _ = factory_tx.send(FactoryResponse::LinkCreated { id });
                }
                Err(e) => {
                    let _ = factory_tx.send(FactoryResponse::Error(e.to_string()));
                }
            }
        }
        FactoryRequest::SetNodeVolume { node_id, volume } => {
            match set_node_volume(&node_proxies_volume, node_id, volume) {
                Ok(()) => {
                    let _ = factory_tx.send(FactoryResponse::VolumeSet { node_id });
                }
                Err(e) => {
                    let _ = factory_tx.send(FactoryResponse::Error(e.to_string()));
                }
            }
        }
        FactoryRequest::SetNodeMute { node_id, muted } => {
            match set_node_mute(&node_proxies_mute, node_id, muted) {
                Ok(()) => {
                    let _ = factory_tx.send(FactoryResponse::MuteSet { node_id });
                }
                Err(e) => {
                    let _ = factory_tx.send(FactoryResponse::Error(e.to_string()));
                }
            }
        }
        FactoryRequest::DestroyNode(id) => {
            let _ = factory_tx.send(FactoryResponse::NodeDestroyed { id });
        }
        FactoryRequest::DestroyLink(id) => {
            let _ = factory_tx.send(FactoryResponse::LinkDestroyed { id });
        }
        FactoryRequest::Shutdown => {
            info!("Factory received shutdown request");
            main_loop_for_shutdown.quit();
        }
    });

    // Keep proxies alive for the duration of the main loop
    let _node_proxies = node_proxies;
    let _link_proxies = link_proxies;

    info!("Starting PipeWire main loop");
    main_loop.run();

    info!("PipeWire runtime exiting");
    Ok(())
}

fn create_virtual_sink(
    core: &pipewire::core::CoreRc,
    props: &VirtualSinkProps,
    proxies: &Rc<RefCell<HashMap<u32, pipewire::node::Node>>>,
) -> PwResult<CreatedNode> {
    info!(name = %props.name, "Creating virtual sink");

    let node_props = properties! {
        "factory.name" => "support.null-audio-sink",
        "node.name" => props.name.as_str(),
        "node.description" => props.description.as_str(),
        "media.class" => "Audio/Sink",
        "audio.channels" => props.channels.to_string().as_str(),
        "audio.position" => props.positions.as_str(),
        "undertone.managed" => "true",
        "node.passive" => "true",
        "session.suspend-timeout-seconds" => "0",
    };

    let proxy = core
        .create_object::<pipewire::node::Node>("adapter", &node_props)
        .map_err(|e| PwError::NodeCreationFailed(format!("Failed to create node: {e}")))?;

    let id = proxy.upcast_ref().id();
    debug!(id, name = %props.name, "Virtual sink created");

    // Store the proxy to keep the node alive
    proxies.borrow_mut().insert(id, proxy);

    Ok(CreatedNode { id, name: props.name.clone() })
}

/// Create a volume filter node (a null-audio-sink configured for volume control).
fn create_volume_filter(
    core: &pipewire::core::CoreRc,
    name: &str,
    description: &str,
    channels: u32,
    proxies: &Rc<RefCell<HashMap<u32, pipewire::node::Node>>>,
) -> PwResult<CreatedNode> {
    info!(name = %name, "Creating volume filter");

    let positions = if channels == 1 { "MONO" } else { "FL,FR" };

    let node_props = properties! {
        "factory.name" => "support.null-audio-sink",
        "node.name" => name,
        "node.description" => description,
        "media.class" => "Audio/Sink",
        "audio.channels" => channels.to_string().as_str(),
        "audio.position" => positions,
        "undertone.managed" => "true",
        "undertone.volume-filter" => "true",
        "node.passive" => "true",
        "session.suspend-timeout-seconds" => "0",
        // Enable monitor channel volumes for volume control
        "monitor.channel-volumes" => "true",
    };

    let proxy = core
        .create_object::<pipewire::node::Node>("adapter", &node_props)
        .map_err(|e| PwError::NodeCreationFailed(format!("Failed to create volume filter: {e}")))?;

    let id = proxy.upcast_ref().id();
    debug!(id, name = %name, "Volume filter created");

    // Store the proxy to keep the node alive and enable volume control
    proxies.borrow_mut().insert(id, proxy);

    Ok(CreatedNode { id, name: name.to_string() })
}

/// Set volume on a node using SPA Props.
fn set_node_volume(
    proxies: &Rc<RefCell<HashMap<u32, pipewire::node::Node>>>,
    node_id: u32,
    volume: f32,
) -> PwResult<()> {
    let proxies = proxies.borrow();
    let node = proxies
        .get(&node_id)
        .ok_or_else(|| PwError::NodeNotFound(format!("Node {} not found in proxies", node_id)))?;

    debug!(node_id, volume, "Setting node volume");

    // Create a Props object with the volume property
    let props_object = Value::Object(Object {
        type_: SpaTypes::ObjectParamProps.as_raw(),
        id: ParamType::Props.as_raw(),
        properties: vec![Property::new(spa_props::SPA_PROP_VOLUME, Value::Float(volume))],
    });

    // Serialize the object to a Pod
    let pod_bytes: Vec<u8> = PodSerializer::serialize(Cursor::new(Vec::new()), &props_object)
        .map_err(|e| {
            PwError::VolumeControlFailed(format!("Failed to serialize volume pod: {e:?}"))
        })?
        .0
        .into_inner();

    // Get the Pod reference
    let pod = Pod::from_bytes(&pod_bytes).ok_or_else(|| {
        PwError::VolumeControlFailed("Failed to create Pod from bytes".to_string())
    })?;

    // Set the param on the node
    node.set_param(ParamType::Props, 0, pod);

    debug!(node_id, volume, "Volume set successfully");
    Ok(())
}

/// Set mute state on a node using SPA Props.
fn set_node_mute(
    proxies: &Rc<RefCell<HashMap<u32, pipewire::node::Node>>>,
    node_id: u32,
    muted: bool,
) -> PwResult<()> {
    let proxies = proxies.borrow();
    let node = proxies
        .get(&node_id)
        .ok_or_else(|| PwError::NodeNotFound(format!("Node {} not found in proxies", node_id)))?;

    debug!(node_id, muted, "Setting node mute");

    // Create a Props object with the mute property
    let props_object = Value::Object(Object {
        type_: SpaTypes::ObjectParamProps.as_raw(),
        id: ParamType::Props.as_raw(),
        properties: vec![Property::new(spa_props::SPA_PROP_MUTE, Value::Bool(muted))],
    });

    // Serialize the object to a Pod
    let pod_bytes: Vec<u8> = PodSerializer::serialize(Cursor::new(Vec::new()), &props_object)
        .map_err(|e| PwError::VolumeControlFailed(format!("Failed to serialize mute pod: {e:?}")))?
        .0
        .into_inner();

    // Get the Pod reference
    let pod = Pod::from_bytes(&pod_bytes).ok_or_else(|| {
        PwError::VolumeControlFailed("Failed to create Pod from bytes".to_string())
    })?;

    // Set the param on the node
    node.set_param(ParamType::Props, 0, pod);

    debug!(node_id, muted, "Mute set successfully");
    Ok(())
}

fn create_link(
    core: &pipewire::core::CoreRc,
    output_node: u32,
    output_port: &str,
    input_node: u32,
    input_port: &str,
    proxies: &Rc<RefCell<Vec<pipewire::link::Link>>>,
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

    // Store the proxy to keep the link alive
    proxies.borrow_mut().push(proxy);

    Ok(id)
}

fn handle_global(
    event_tx: &mpsc::Sender<GraphEvent>,
    graph: &GraphManager,
    nodes: &Rc<RefCell<HashMap<u32, String>>>,
    global: &GlobalObject<&DictRef>,
) {
    let props = global.props.as_ref();

    match global.type_ {
        ObjectType::Node => {
            let name = props.and_then(|p| p.get("node.name")).unwrap_or("unknown").to_string();

            let description = props.and_then(|p| p.get("node.description")).map(String::from);
            let media_class = props.and_then(|p| p.get("media.class")).map(String::from);
            let app_name = props.and_then(|p| p.get("application.name")).map(String::from);
            let binary = props.and_then(|p| p.get("application.process.binary")).map(String::from);
            let pid =
                props.and_then(|p| p.get("application.process.id")).and_then(|s| s.parse().ok());

            let is_undertone =
                name.starts_with("ut-") || props.and_then(|p| p.get("undertone.managed")).is_some();

            let node_info = NodeInfo {
                id: global.id,
                name: name.clone(),
                description,
                media_class: media_class.clone(),
                application_name: app_name.clone(),
                binary_name: binary,
                pid,
                is_undertone_managed: is_undertone,
                properties: props
                    .map(|p| p.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
                    .unwrap_or_default(),
            };

            debug!(id = global.id, name = %name, class = ?media_class, "Node added");

            nodes.borrow_mut().insert(global.id, name.clone());
            graph.add_node(node_info.clone());
            let _ = event_tx.blocking_send(GraphEvent::NodeAdded(node_info.clone()));

            // Detect Wave:3
            if node_info.is_wave3() && node_info.name == "wave3-source" {
                info!("Wave:3 microphone detected");
                let serial =
                    props.and_then(|p| p.get("device.serial")).unwrap_or("unknown").to_string();
                let _ = event_tx.blocking_send(GraphEvent::Wave3Detected { serial });
            }

            // Detect audio clients
            if media_class.as_deref() == Some("Stream/Output/Audio") && !is_undertone {
                let _ = event_tx.blocking_send(GraphEvent::ClientAppeared {
                    id: global.id,
                    name: app_name.unwrap_or_else(|| name.clone()),
                    pid,
                });
            }
        }

        ObjectType::Port => {
            let name = props.and_then(|p| p.get("port.name")).unwrap_or("unknown").to_string();

            let node_id =
                props.and_then(|p| p.get("node.id")).and_then(|s| s.parse().ok()).unwrap_or(0);

            let direction = props
                .and_then(|p| p.get("port.direction"))
                .map(|d| if d == "in" { PortDirection::Input } else { PortDirection::Output })
                .unwrap_or(PortDirection::Output);

            let channel = props.and_then(|p| p.get("audio.channel")).map(String::from);

            let port_info = PortInfo { id: global.id, name, direction, node_id, channel };

            graph.add_port(port_info.clone());
            let _ = event_tx.blocking_send(GraphEvent::PortAdded(port_info));
        }

        ObjectType::Link => {
            let output_node = props
                .and_then(|p| p.get("link.output.node"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let input_node = props
                .and_then(|p| p.get("link.input.node"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            debug!(id = global.id, output_node, input_node, "Link created");

            let _ = event_tx.blocking_send(GraphEvent::LinkCreated {
                id: global.id,
                output_node,
                input_node,
            });
        }

        _ => {}
    }
}

fn handle_global_remove(
    event_tx: &mpsc::Sender<GraphEvent>,
    nodes: &Rc<RefCell<HashMap<u32, String>>>,
    id: u32,
) {
    if let Some(name) = nodes.borrow_mut().remove(&id) {
        debug!(id, name = %name, "Node removed");

        if name == "wave3-source" {
            warn!("Wave:3 microphone disconnected");
            let _ = event_tx.blocking_send(GraphEvent::Wave3Removed);
        }

        let _ = event_tx.blocking_send(GraphEvent::NodeRemoved { id, name });
    } else {
        let _ = event_tx.blocking_send(GraphEvent::LinkRemoved { id });
    }
}
