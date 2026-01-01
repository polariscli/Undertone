//! Undertone Daemon - PipeWire audio control service.
//!
//! This is the main entry point for the Undertone daemon, which manages
//! PipeWire audio routing, persistence, and Wave:3 hardware integration.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

mod config;
mod server;
mod signals;

use undertone_core::channel::ChannelState;
use undertone_core::state::{DaemonState, StateSnapshot};
use undertone_db::Database;
use undertone_ipc::{
    AppDiscoveredData, ChannelMuteChangedData, ChannelVolumeChangedData, DeviceConnectedData,
    Event, EventType, IpcServer, socket_path,
};
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

    info!(version = env!("CARGO_PKG_VERSION"), "Starting Undertone daemon");

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
    let mut routes = db.load_routes().context("Failed to load routes")?;
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

    // Create volume filter nodes for each channel
    info!("Creating volume filter nodes...");
    match pw_runtime.create_channel_volume_filters(DEFAULT_CHANNELS) {
        Ok(created) => {
            info!(count = created.len(), "Created volume filter nodes");
            for (name, id) in &created {
                graph.record_created_node(name.clone(), *id);
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to create volume filter nodes");
        }
    }

    // Wait for ports to be discovered before creating links
    info!("Waiting for port discovery...");
    sleep(Duration::from_millis(500)).await;

    // Create links from channels through volume filters to mix nodes
    info!("Creating channel-to-mix links with volume filters...");
    match pw_runtime.create_channel_to_mix_links_with_filters() {
        Ok(created) => {
            info!(count = created.len(), "Created channel-to-mix links with volume filters");
            for (description, id) in &created {
                graph.record_created_link(description.clone(), *id);
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to create channel-to-mix links");
        }
    }

    // Link monitor-mix to headphones if Wave:3 is connected
    if graph.get_node_by_name("wave3-sink").is_some() {
        info!("Linking monitor-mix to Wave:3 headphones...");
        match pw_runtime.link_monitor_to_headphones() {
            Ok((left_id, right_id)) => {
                info!("Monitor-mix linked to headphones");
                graph.record_created_link("monitor-mix->wave3-sink:FL".to_string(), left_id);
                graph.record_created_link("monitor-mix->wave3-sink:FR".to_string(), right_id);
            }
            Err(e) => {
                warn!(error = %e, "Failed to link monitor-mix to headphones (Wave:3 may not be connected)");
            }
        }
    } else {
        info!("Wave:3 sink not found - skipping monitor-mix to headphones link");
    }

    // Start IPC server
    let socket = socket_path();
    info!(?socket, "Starting IPC server");
    let (ipc_server, mut request_rx) =
        IpcServer::bind(&socket).await.context("Failed to start IPC server")?;

    // Get event sender for broadcasting events to IPC clients
    let event_tx = ipc_server.event_sender();

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
                        device_serial = Some(serial.clone());
                        state = DaemonState::Running;

                        // Emit IPC event
                        let _ = event_tx.send(Event {
                            event: EventType::DeviceConnected,
                            data: serde_json::to_value(DeviceConnectedData {
                                serial: serial.clone(),
                            })
                            .unwrap_or_default(),
                        });

                        // Try to link monitor-mix to headphones now that Wave:3 is connected
                        if !graph.get_created_links().contains_key("monitor-mix->wave3-sink:FL") {
                            match pw_runtime.link_monitor_to_headphones() {
                                Ok((left_id, right_id)) => {
                                    info!("Monitor-mix linked to Wave:3 headphones");
                                    graph.record_created_link("monitor-mix->wave3-sink:FL".to_string(), left_id);
                                    graph.record_created_link("monitor-mix->wave3-sink:FR".to_string(), right_id);
                                }
                                Err(e) => {
                                    warn!(error = %e, "Failed to link monitor-mix to headphones");
                                }
                            }
                        }
                    }

                    GraphEvent::Wave3Removed => {
                        warn!("Wave:3 disconnected");
                        device_connected = false;
                        device_serial = None;
                        state = DaemonState::DeviceDisconnected;

                        // Emit IPC event
                        let _ = event_tx.send(Event {
                            event: EventType::DeviceDisconnected,
                            data: serde_json::json!({}),
                        });
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

                        // Get the app's binary name from the graph if available
                        let binary_name = graph.get_node(id).and_then(|n| n.binary_name.clone());

                        // Find the target channel based on routing rules
                        let target_channel = undertone_core::routing::find_channel_for_app(
                            &name,
                            binary_name.as_deref(),
                            &routes,
                        );

                        info!(
                            app_id = id,
                            app_name = %name,
                            channel = %target_channel,
                            "Routing new app to channel"
                        );

                        // Route the app to the target channel
                        match pw_runtime.route_app_to_channel(id, &target_channel) {
                            Ok(link_ids) => {
                                debug!(
                                    app_id = id,
                                    links_created = link_ids.len(),
                                    "App routed successfully"
                                );
                            }
                            Err(e) => {
                                warn!(
                                    app_id = id,
                                    error = %e,
                                    "Failed to route app (may be transient)"
                                );
                            }
                        }

                        // Emit IPC event
                        let _ = event_tx.send(Event {
                            event: EventType::AppDiscovered,
                            data: serde_json::to_value(AppDiscoveredData {
                                app_id: id,
                                name: name.clone(),
                                binary: binary_name,
                                pid,
                                channel: target_channel,
                            })
                            .unwrap_or_default(),
                        });
                    }

                    GraphEvent::ClientDisappeared { id } => {
                        debug!(id, "Audio client disappeared");

                        // Emit IPC event
                        let _ = event_tx.send(Event {
                            event: EventType::AppRemoved,
                            data: serde_json::json!({ "app_id": id }),
                        });
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
                    app_routes: vec![], // TODO: Track active app routes
                    mixer: Default::default(),
                    active_profile: "Default".to_string(),
                    created_nodes: graph.get_created_nodes(),
                    created_links: graph.get_created_links(),
                };

                let handle_result = server::handle_request(&request.method, &snapshot);
                let response = undertone_ipc::Response {
                    id: request.id,
                    result: handle_result.response,
                };
                let _ = response_tx.send(response).await;

                // Process command if one was returned
                if let Some(cmd) = handle_result.command {
                    use undertone_core::Command;
                    use undertone_core::mixer::MixType;

                    match cmd {
                        Command::SetChannelVolume { channel, mix, volume } => {
                            if let Some(ch) = channels.iter_mut().find(|c| c.config.name == channel) {
                                match mix {
                                    MixType::Stream => ch.stream_volume = volume,
                                    MixType::Monitor => ch.monitor_volume = volume,
                                }
                                info!(channel = %channel, ?mix, volume, "Channel volume updated");

                                // Apply to PipeWire volume filter node
                                let filter_name = match mix {
                                    MixType::Stream => format!("ut-ch-{channel}-stream-vol"),
                                    MixType::Monitor => format!("ut-ch-{channel}-monitor-vol"),
                                };
                                if let Some(node_id) = graph.get_created_node_id(&filter_name) {
                                    if let Err(e) = pw_runtime.set_node_volume(node_id, volume) {
                                        error!(error = %e, filter = %filter_name, "Failed to set volume on filter node");
                                    } else {
                                        debug!(filter = %filter_name, volume, "Volume applied to PipeWire");
                                    }
                                } else {
                                    warn!(filter = %filter_name, "Volume filter node not found");
                                }

                                // Emit event
                                let _ = event_tx.send(Event {
                                    event: EventType::ChannelVolumeChanged,
                                    data: serde_json::to_value(ChannelVolumeChangedData {
                                        channel: channel.clone(),
                                        mix,
                                        volume,
                                    }).unwrap_or_default(),
                                });
                            }
                        }

                        Command::SetChannelMute { channel, mix, muted } => {
                            if let Some(ch) = channels.iter_mut().find(|c| c.config.name == channel) {
                                match mix {
                                    MixType::Stream => ch.stream_muted = muted,
                                    MixType::Monitor => ch.monitor_muted = muted,
                                }
                                info!(channel = %channel, ?mix, muted, "Channel mute updated");

                                // Apply to PipeWire volume filter node
                                let filter_name = match mix {
                                    MixType::Stream => format!("ut-ch-{channel}-stream-vol"),
                                    MixType::Monitor => format!("ut-ch-{channel}-monitor-vol"),
                                };
                                if let Some(node_id) = graph.get_created_node_id(&filter_name) {
                                    if let Err(e) = pw_runtime.set_node_mute(node_id, muted) {
                                        error!(error = %e, filter = %filter_name, "Failed to set mute on filter node");
                                    } else {
                                        debug!(filter = %filter_name, muted, "Mute applied to PipeWire");
                                    }
                                } else {
                                    warn!(filter = %filter_name, "Volume filter node not found");
                                }

                                // Emit event
                                let _ = event_tx.send(Event {
                                    event: EventType::ChannelMuteChanged,
                                    data: serde_json::to_value(ChannelMuteChangedData {
                                        channel: channel.clone(),
                                        mix,
                                        muted,
                                    }).unwrap_or_default(),
                                });
                            }
                        }

                        Command::SetAppRoute { app_pattern, channel } => {
                            use undertone_core::routing::{PatternType, RouteRule};

                            // Update in-memory routes
                            routes.retain(|r| r.pattern != app_pattern);
                            let rule = RouteRule {
                                pattern: app_pattern.clone(),
                                pattern_type: PatternType::Exact,
                                channel: channel.clone(),
                                priority: 100,
                            };
                            routes.push(rule.clone());
                            info!(app_pattern = %app_pattern, channel = %channel, "App route set");

                            // Save to database
                            if let Err(e) = db.save_route(&rule) {
                                error!(error = %e, "Failed to save route to database");
                            }

                            // Apply routing to matching active apps
                            let audio_clients = pw_runtime.get_audio_clients();
                            for client in audio_clients {
                                // Check if this client matches the pattern
                                let matches = client.application_name.as_ref().map_or(false, |name| {
                                    rule.matches(name)
                                }) || client.binary_name.as_ref().map_or(false, |name| {
                                    rule.matches(name)
                                }) || rule.matches(&client.name);

                                if matches {
                                    info!(
                                        app_id = client.id,
                                        app_name = %client.name,
                                        channel = %channel,
                                        "Re-routing matching app"
                                    );
                                    match pw_runtime.route_app_to_channel(client.id, &channel) {
                                        Ok(link_ids) => {
                                            debug!(
                                                app_id = client.id,
                                                links_created = link_ids.len(),
                                                "App re-routed successfully"
                                            );
                                        }
                                        Err(e) => {
                                            warn!(
                                                app_id = client.id,
                                                error = %e,
                                                "Failed to re-route app"
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        Command::RemoveAppRoute { app_pattern } => {
                            routes.retain(|r| r.pattern != app_pattern);
                            info!(app_pattern = %app_pattern, "App route removed");

                            // Remove from database
                            if let Err(e) = db.delete_route(&app_pattern) {
                                error!(error = %e, "Failed to remove route from database");
                            }
                        }

                        Command::SaveProfile { name } => {
                            // TODO: Implement profile saving
                            info!(name = %name, "Profile saving not yet implemented");
                        }

                        Command::LoadProfile { name } => {
                            // TODO: Implement profile loading
                            info!(name = %name, "Profile loading not yet implemented");
                        }

                        Command::DeleteProfile { name } => {
                            // TODO: Implement profile deletion
                            info!(name = %name, "Profile deletion not yet implemented");
                        }

                        Command::SetMicGain { gain } => {
                            // TODO: Apply via ALSA fallback or HID
                            info!(gain, "Mic gain setting not yet implemented");
                        }

                        Command::SetMicMute { muted } => {
                            // TODO: Apply via ALSA fallback or HID
                            info!(muted, "Mic mute setting not yet implemented");
                        }

                        Command::Reconcile => {
                            state = DaemonState::Reconciling;
                            // TODO: Implement full reconciliation
                            info!("Reconciliation triggered");
                            state = DaemonState::Running;
                        }

                        Command::Shutdown => {
                            info!("Shutdown command processed");
                            break;
                        }
                    }
                }
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
