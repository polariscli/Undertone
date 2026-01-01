//! Undertone Daemon - PipeWire audio control service.
//!
//! This is the main entry point for the Undertone daemon, which manages
//! PipeWire audio routing, persistence, and Wave:3 hardware integration.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

mod config;
mod server;
mod signals;

use undertone_core::channel::ChannelState;
use undertone_core::state::{DaemonState, StateSnapshot};
use undertone_db::Database;
use undertone_ipc::{socket_path, IpcServer};
use undertone_pipewire::{GraphEvent, GraphManager, PipeWireRuntime};

/// Default channels to create
const DEFAULT_CHANNELS: &[&str] = &["system", "voice", "music", "browser", "game"];

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("undertone=info".parse()?)
                .add_directive("undertone_daemon=debug".parse()?)
                .add_directive("undertone_pipewire=debug".parse()?),
        )
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting Undertone daemon"
    );

    // Load configuration
    let config = config::load_config()?;
    info!("Configuration loaded");

    // Open database
    let db = Database::open().context("Failed to open database")?;
    info!("Database initialized");

    // Load channels from database
    let mut channels: Vec<ChannelState> = db.load_channels().context("Failed to load channels")?;
    info!(count = channels.len(), "Loaded channels from database");

    // Load routing rules
    let routes = db.load_routes().context("Failed to load routes")?;
    info!(count = routes.len(), "Loaded routing rules");

    // Initialize PipeWire graph manager
    let graph = Arc::new(GraphManager::new());

    // Spawn PipeWire runtime
    info!("Starting PipeWire runtime...");
    let (pw_runtime, mut graph_event_rx) = PipeWireRuntime::spawn(Arc::clone(&graph));

    // Wait for PipeWire connection
    info!("Waiting for PipeWire connection...");
    let mut connected = false;
    while let Some(event) = graph_event_rx.recv().await {
        if matches!(event, GraphEvent::Connected) {
            connected = true;
            info!("PipeWire connected!");
            break;
        }
    }

    if !connected {
        error!("Failed to connect to PipeWire");
        return Err(anyhow::anyhow!("PipeWire connection failed"));
    }

    // Give PipeWire a moment to enumerate existing nodes
    sleep(Duration::from_millis(500)).await;

    // Create virtual channel sinks
    info!("Creating virtual channel sinks...");
    match pw_runtime.create_channel_sinks(DEFAULT_CHANNELS) {
        Ok(created) => {
            info!(count = created.len(), "Created channel sinks");
            for node in &created {
                graph.record_created_node(node.name.clone(), node.id);
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to create channel sinks");
            // Continue anyway - we might be able to recover
        }
    }

    // Create mix nodes
    info!("Creating mix nodes...");
    match pw_runtime.create_mix_nodes() {
        Ok(created) => {
            info!(count = created.len(), "Created mix nodes");
            for node in &created {
                graph.record_created_node(node.name.clone(), node.id);
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to create mix nodes");
        }
    }

    // Start IPC server
    let socket = socket_path();
    info!(?socket, "Starting IPC server");
    let (ipc_server, mut request_rx) = IpcServer::bind(&socket)
        .await
        .context("Failed to start IPC server")?;

    // Spawn IPC server task
    let ipc_handle = tokio::spawn(async move {
        ipc_server.run().await;
    });

    // Set up signal handling
    let mut shutdown_rx = signals::setup_signal_handlers()?;

    // Build initial state snapshot
    let mut state = DaemonState::Running;
    let mut device_connected = false;
    let mut device_serial: Option<String> = None;

    info!("Daemon running. Press Ctrl+C to exit.");

    // Main event loop
    loop {
        tokio::select! {
            // Handle PipeWire graph events
            Some(event) = graph_event_rx.recv() => {
                match event {
                    GraphEvent::Connected => {
                        info!("PipeWire reconnected");
                        state = DaemonState::Reconciling;
                        // TODO: Trigger reconciliation
                    }

                    GraphEvent::Disconnected => {
                        warn!("PipeWire disconnected");
                        state = DaemonState::Error("PipeWire disconnected".to_string());
                    }

                    GraphEvent::Wave3Detected { serial } => {
                        info!(serial = %serial, "Wave:3 detected");
                        device_connected = true;
                        device_serial = Some(serial);
                        state = DaemonState::Running;
                    }

                    GraphEvent::Wave3Removed => {
                        warn!("Wave:3 disconnected");
                        device_connected = false;
                        device_serial = None;
                        state = DaemonState::DeviceDisconnected;
                    }

                    GraphEvent::NodeAdded(node) => {
                        debug!(id = node.id, name = %node.name, "Node added to graph");
                        // Node is already added to graph by the PipeWire thread
                    }

                    GraphEvent::NodeRemoved { id, name } => {
                        debug!(id, name = %name, "Node removed from graph");
                        graph.remove_node(id);

                        // Check if one of our nodes was removed
                        if name.starts_with("ut-") {
                            warn!(name = %name, "Undertone node was removed - may need reconciliation");
                        }
                    }

                    GraphEvent::PortAdded(port) => {
                        debug!(id = port.id, name = %port.name, node_id = port.node_id, "Port added");
                    }

                    GraphEvent::PortRemoved { id } => {
                        debug!(id, "Port removed");
                        graph.remove_port(id);
                    }

                    GraphEvent::LinkCreated { id, output_node, input_node } => {
                        debug!(id, output_node, input_node, "Link created");
                    }

                    GraphEvent::LinkRemoved { id } => {
                        debug!(id, "Link removed");
                    }

                    GraphEvent::ClientAppeared { id, name, pid } => {
                        info!(id, name = %name, pid = ?pid, "Audio client appeared");
                        // TODO: Apply routing rules
                    }

                    GraphEvent::ClientDisappeared { id } => {
                        debug!(id, "Audio client disappeared");
                    }
                }
            }

            // Handle IPC requests
            Some((client_id, request, response_tx)) = request_rx.recv() => {
                debug!(client_id, request_id = request.id, "Handling IPC request");

                // Build current state snapshot
                let snapshot = StateSnapshot {
                    state: state.clone(),
                    device_connected,
                    device_serial: device_serial.clone(),
                    channels: channels.clone(),
                    app_routes: vec![], // TODO: Track active routes
                    mixer: Default::default(),
                    active_profile: "Default".to_string(),
                    created_nodes: graph.get_created_nodes(),
                    created_links: Default::default(),
                };

                let result = server::handle_request(&request.method, &snapshot);
                let response = undertone_ipc::Response {
                    id: request.id,
                    result,
                };
                let _ = response_tx.send(response).await;
            }

            // Handle shutdown signal
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    // Cleanup
    info!("Shutting down...");
    pw_runtime.shutdown();
    ipc_handle.abort();

    info!("Undertone daemon stopped");
    Ok(())
}
