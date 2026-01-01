//! PipeWire graph monitoring.
//!
//! This module handles the connection to PipeWire and monitors the audio graph
//! for changes. It runs in a dedicated thread because PipeWire is not thread-safe.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use pipewire::context::ContextBox;
use pipewire::main_loop::MainLoopBox;
use pipewire::registry::GlobalObject;
use pipewire::spa::utils::dict::DictRef;
use pipewire::types::ObjectType;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{PwError, PwResult};
use crate::graph::GraphManager;
use crate::node::{NodeInfo, PortDirection, PortInfo};

/// Events emitted by the graph monitor.
#[derive(Debug, Clone)]
pub enum GraphEvent {
    /// PipeWire connection established
    Connected,
    /// PipeWire connection lost
    Disconnected,
    /// A node was added to the graph
    NodeAdded(NodeInfo),
    /// A node was removed from the graph
    NodeRemoved { id: u32, name: String },
    /// A port was added
    PortAdded(PortInfo),
    /// A port was removed
    PortRemoved { id: u32 },
    /// A link was created
    LinkCreated { id: u32, output_node: u32, input_node: u32 },
    /// A link was removed
    LinkRemoved { id: u32 },
    /// Wave:3 device detected
    Wave3Detected { serial: String },
    /// Wave:3 device removed
    Wave3Removed,
    /// An audio client appeared
    ClientAppeared { id: u32, name: String, pid: Option<u32> },
    /// An audio client disappeared
    ClientDisappeared { id: u32 },
}

/// Monitors the PipeWire graph for changes.
pub struct GraphMonitor {
    graph: Arc<GraphManager>,
    event_tx: mpsc::Sender<GraphEvent>,
}

impl GraphMonitor {
    /// Create a new graph monitor.
    #[must_use]
    pub fn new(graph: Arc<GraphManager>, event_tx: mpsc::Sender<GraphEvent>) -> Self {
        Self { graph, event_tx }
    }

    /// Start monitoring the PipeWire graph.
    ///
    /// This function blocks and should be run in a dedicated thread.
    pub fn run(&self) -> PwResult<()> {
        // Initialize PipeWire
        pipewire::init();

        info!("Connecting to PipeWire...");

        // Create the main loop
        let main_loop = MainLoopBox::new(None)
            .map_err(|e| PwError::MainLoopError(format!("Failed to create main loop: {e}")))?;

        let context = ContextBox::new(main_loop.loop_(), None)
            .map_err(|e| PwError::ConnectionFailed(format!("Failed to create context: {e}")))?;

        let core = context
            .connect(None)
            .map_err(|e| PwError::ConnectionFailed(format!("Failed to connect: {e}")))?;

        let registry = core.get_registry()
            .map_err(|e| PwError::RegistryError(format!("Failed to get registry: {e}")))?;

        info!("Connected to PipeWire");

        // Send connected event
        let _ = self.event_tx.blocking_send(GraphEvent::Connected);

        // Track nodes for removal events
        let nodes: Rc<RefCell<HashMap<u32, String>>> = Rc::new(RefCell::new(HashMap::new()));
        let nodes_clone = Rc::clone(&nodes);

        // Set up registry listener
        let event_tx_global = self.event_tx.clone();
        let event_tx_remove = self.event_tx.clone();
        let graph = Arc::clone(&self.graph);

        let _listener = registry
            .add_listener_local()
            .global(move |global| {
                Self::handle_global(&event_tx_global, &graph, &nodes, global);
            })
            .global_remove(move |id| {
                Self::handle_global_remove(&event_tx_remove, &nodes_clone, id);
            })
            .register();

        // Run the main loop
        info!("Starting PipeWire main loop");
        main_loop.run();

        info!("PipeWire main loop exited");
        Ok(())
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

                let is_undertone = name.starts_with("ut-");

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
                        .map(|p| {
                            p.iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                };

                debug!(id = global.id, name = %name, class = ?media_class, "Node added");

                // Track node for removal
                nodes.borrow_mut().insert(global.id, name.clone());

                // Add to graph
                graph.add_node(node_info.clone());

                // Send event
                let _ = event_tx.blocking_send(GraphEvent::NodeAdded(node_info.clone()));

                // Check for Wave:3
                if node_info.is_wave3() && node_info.name == "wave3-source" {
                    info!("Wave:3 microphone detected");
                    let serial = props
                        .and_then(|p| p.get("device.serial"))
                        .unwrap_or("unknown")
                        .to_string();
                    let _ = event_tx.blocking_send(GraphEvent::Wave3Detected { serial });
                }

                // Check for audio client
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
                    .map(|d| {
                        if d == "in" {
                            PortDirection::Input
                        } else {
                            PortDirection::Output
                        }
                    })
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
        // Check if this was a node we tracked
        if let Some(name) = nodes.borrow_mut().remove(&id) {
            debug!(id, name = %name, "Node removed");

            // Check if Wave:3 was removed
            if name == "wave3-source" {
                warn!("Wave:3 microphone disconnected");
                let _ = event_tx.blocking_send(GraphEvent::Wave3Removed);
            }

            let _ = event_tx.blocking_send(GraphEvent::NodeRemoved { id, name });
        } else {
            // Could be a link or port
            let _ = event_tx.blocking_send(GraphEvent::LinkRemoved { id });
        }
    }
}

/// Spawn the graph monitor in a dedicated thread.
pub fn spawn_monitor(graph: Arc<GraphManager>) -> mpsc::Receiver<GraphEvent> {
    let (tx, rx) = mpsc::channel(256);

    std::thread::Builder::new()
        .name("pipewire-monitor".to_string())
        .spawn(move || {
            let monitor = GraphMonitor::new(graph, tx);
            if let Err(e) = monitor.run() {
                error!(error = %e, "Graph monitor failed");
            }
        })
        .expect("Failed to spawn PipeWire monitor thread");

    rx
}
