//! PipeWire runtime that combines monitoring and node creation.
//!
//! This module provides a unified runtime that handles both graph monitoring
//! and node creation in a single PipeWire thread.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc as std_mpsc;
use std::sync::Arc;
use std::time::Duration;

use pipewire::context::ContextRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::properties::properties;
use pipewire::proxy::ProxyT;
use pipewire::registry::GlobalObject;
use pipewire::spa::utils::dict::DictRef;
use pipewire::types::ObjectType;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{PwError, PwResult};
use crate::factory::{CreatedNode, FactoryRequest, FactoryResponse};
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

        (
            Self {
                factory_tx,
                factory_rx,
                graph,
            },
            event_rx,
        )
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
    let node_proxies: Rc<RefCell<Vec<pipewire::node::Node>>> = Rc::new(RefCell::new(Vec::new()));
    let link_proxies: Rc<RefCell<Vec<pipewire::link::Link>>> = Rc::new(RefCell::new(Vec::new()));
    let node_proxies_clone = Rc::clone(&node_proxies);
    let link_proxies_clone = Rc::clone(&link_proxies);

    // Attach factory request receiver to the loop
    let _factory_receiver = factory_rx.attach(main_loop.loop_(), move |request| {
        match request {
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
            FactoryRequest::CreateLink { output_node, output_port, input_node, input_port } => {
                match create_link(&core_for_factory, output_node, &output_port, input_node, &input_port, &link_proxies_clone) {
                    Ok(id) => {
                        let _ = factory_tx.send(FactoryResponse::LinkCreated { id });
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
    proxies: &Rc<RefCell<Vec<pipewire::node::Node>>>,
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
    proxies.borrow_mut().push(proxy);

    Ok(CreatedNode {
        id,
        name: props.name.clone(),
    })
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
            let name = props
                .and_then(|p| p.get("node.name"))
                .unwrap_or("unknown")
                .to_string();

            let description = props.and_then(|p| p.get("node.description")).map(String::from);
            let media_class = props.and_then(|p| p.get("media.class")).map(String::from);
            let app_name = props.and_then(|p| p.get("application.name")).map(String::from);
            let binary = props.and_then(|p| p.get("application.process.binary")).map(String::from);
            let pid = props
                .and_then(|p| p.get("application.process.id"))
                .and_then(|s| s.parse().ok());

            let is_undertone = name.starts_with("ut-") ||
                props.and_then(|p| p.get("undertone.managed")).is_some();

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
                let serial = props
                    .and_then(|p| p.get("device.serial"))
                    .unwrap_or("unknown")
                    .to_string();
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
            let name = props
                .and_then(|p| p.get("port.name"))
                .unwrap_or("unknown")
                .to_string();

            let node_id = props
                .and_then(|p| p.get("node.id"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let direction = props
                .and_then(|p| p.get("port.direction"))
                .map(|d| if d == "in" { PortDirection::Input } else { PortDirection::Output })
                .unwrap_or(PortDirection::Output);

            let channel = props.and_then(|p| p.get("audio.channel")).map(String::from);

            let port_info = PortInfo {
                id: global.id,
                name,
                direction,
                node_id,
                channel,
            };

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
