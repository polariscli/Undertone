//! State reconciliation for self-healing.

use std::sync::Arc;
use tracing::info;

use undertone_core::channel::ChannelConfig;

use crate::graph::GraphManager;
use crate::node::VirtualSinkProps;

/// Actions that the reconciler determines need to be taken.
#[derive(Debug, Clone)]
pub enum ReconcileAction {
    /// Create a virtual sink node
    CreateSink(VirtualSinkProps),
    /// Create a link between nodes
    CreateLink { output_node: String, output_port: String, input_node: String, input_port: String },
    /// Destroy a node
    DestroyNode(u32),
    /// Destroy a link
    DestroyLink(u32),
    /// Log a warning about unexpected state
    Warn(String),
}

/// Reconciler that compares desired state vs actual state.
pub struct Reconciler {
    graph: Arc<GraphManager>,
}

impl Reconciler {
    /// Create a new reconciler.
    #[must_use]
    pub fn new(graph: Arc<GraphManager>) -> Self {
        Self { graph }
    }

    /// Compute actions needed to reach desired state.
    ///
    /// This compares the current `PipeWire` graph state against what
    /// we expect to exist and returns a list of actions to take.
    #[must_use]
    pub fn reconcile(&self, channels: &[ChannelConfig]) -> Vec<ReconcileAction> {
        let mut actions = Vec::new();

        // Check that all channel sinks exist
        for channel in channels {
            let node_name = channel.node_name();
            if self.graph.get_node_by_name(&node_name).is_none() {
                info!(name = %node_name, "Channel node missing, will create");
                actions.push(ReconcileAction::CreateSink(VirtualSinkProps::stereo(
                    &node_name,
                    &format!("Undertone: {} Channel", channel.display_name),
                )));
            }
        }

        // Check that mix nodes exist
        let required_mixes = [
            ("ut-stream-mix", "Undertone: Stream Mix"),
            ("ut-stream-out", "Undertone: Stream Output"),
            ("ut-monitor-mix", "Undertone: Monitor Mix"),
        ];

        for (name, description) in required_mixes {
            if self.graph.get_node_by_name(name).is_none() {
                info!(name, "Mix node missing, will create");
                actions
                    .push(ReconcileAction::CreateSink(VirtualSinkProps::stereo(name, description)));
            }
        }

        // Check that volume filter nodes exist for each channel
        for channel in channels {
            let stream_vol_name = channel.stream_vol_node_name();
            let monitor_vol_name = channel.monitor_vol_node_name();

            if self.graph.get_node_by_name(&stream_vol_name).is_none() {
                info!(name = %stream_vol_name, "Stream volume node missing, will create");
                actions.push(ReconcileAction::CreateSink(VirtualSinkProps::stereo(
                    &stream_vol_name,
                    &format!("Undertone: {} Stream Volume", channel.display_name),
                )));
            }

            if self.graph.get_node_by_name(&monitor_vol_name).is_none() {
                info!(name = %monitor_vol_name, "Monitor volume node missing, will create");
                actions.push(ReconcileAction::CreateSink(VirtualSinkProps::stereo(
                    &monitor_vol_name,
                    &format!("Undertone: {} Monitor Volume", channel.display_name),
                )));
            }
        }

        // Check Wave:3 nodes exist
        let wave3_sink = self.graph.find_wave3_sink();
        if wave3_sink.is_none() {
            actions.push(ReconcileAction::Warn(
                "Wave:3 sink not found - device may be disconnected".to_string(),
            ));
        }

        // Check that links from volume nodes to mix nodes exist
        let stream_mix = self.graph.get_node_by_name("ut-stream-mix");
        let monitor_mix = self.graph.get_node_by_name("ut-monitor-mix");

        for channel in channels {
            let stream_vol_name = channel.stream_vol_node_name();
            let monitor_vol_name = channel.monitor_vol_node_name();

            // Check stream volume → stream mix link
            if let (Some(vol_node), Some(mix_node)) =
                (self.graph.get_node_by_name(&stream_vol_name), stream_mix.as_ref())
                && !self.graph.has_link(vol_node.id, mix_node.id)
            {
                info!(from = %stream_vol_name, to = "ut-stream-mix", "Link missing");
                actions.push(ReconcileAction::CreateLink {
                    output_node: stream_vol_name.clone(),
                    output_port: "monitor_FL".to_string(),
                    input_node: "ut-stream-mix".to_string(),
                    input_port: "playback_FL".to_string(),
                });
                actions.push(ReconcileAction::CreateLink {
                    output_node: stream_vol_name,
                    output_port: "monitor_FR".to_string(),
                    input_node: "ut-stream-mix".to_string(),
                    input_port: "playback_FR".to_string(),
                });
            }

            // Check monitor volume → monitor mix link
            if let (Some(vol_node), Some(mix_node)) =
                (self.graph.get_node_by_name(&monitor_vol_name), monitor_mix.as_ref())
                && !self.graph.has_link(vol_node.id, mix_node.id)
            {
                info!(from = %monitor_vol_name, to = "ut-monitor-mix", "Link missing");
                actions.push(ReconcileAction::CreateLink {
                    output_node: monitor_vol_name.clone(),
                    output_port: "monitor_FL".to_string(),
                    input_node: "ut-monitor-mix".to_string(),
                    input_port: "playback_FL".to_string(),
                });
                actions.push(ReconcileAction::CreateLink {
                    output_node: monitor_vol_name,
                    output_port: "monitor_FR".to_string(),
                    input_node: "ut-monitor-mix".to_string(),
                    input_port: "playback_FR".to_string(),
                });
            }
        }

        // Check that links from mixes to outputs exist
        // Stream mix → Stream output (for OBS capture)
        if let (Some(stream_mix_node), Some(stream_out)) =
            (stream_mix.as_ref(), self.graph.get_node_by_name("ut-stream-out"))
            && !self.graph.has_link(stream_mix_node.id, stream_out.id)
        {
            info!(from = "ut-stream-mix", to = "ut-stream-out", "Link missing");
            actions.push(ReconcileAction::CreateLink {
                output_node: "ut-stream-mix".to_string(),
                output_port: "monitor_FL".to_string(),
                input_node: "ut-stream-out".to_string(),
                input_port: "playback_FL".to_string(),
            });
            actions.push(ReconcileAction::CreateLink {
                output_node: "ut-stream-mix".to_string(),
                output_port: "monitor_FR".to_string(),
                input_node: "ut-stream-out".to_string(),
                input_port: "playback_FR".to_string(),
            });
        }

        // Monitor mix → Wave:3 sink (headphones)
        if let (Some(monitor_mix_node), Some(wave3)) = (monitor_mix.as_ref(), wave3_sink)
            && !self.graph.has_link(monitor_mix_node.id, wave3.id)
        {
            info!(from = "ut-monitor-mix", to = %wave3.name, "Link missing");
            actions.push(ReconcileAction::CreateLink {
                output_node: "ut-monitor-mix".to_string(),
                output_port: "monitor_FL".to_string(),
                input_node: wave3.name.clone(),
                input_port: "playback_FL".to_string(),
            });
            actions.push(ReconcileAction::CreateLink {
                output_node: "ut-monitor-mix".to_string(),
                output_port: "monitor_FR".to_string(),
                input_node: wave3.name,
                input_port: "playback_FR".to_string(),
            });
        }

        actions
    }

    /// Check if the graph is in the expected state.
    #[must_use]
    pub fn is_healthy(&self, channels: &[ChannelConfig]) -> bool {
        self.reconcile(channels).is_empty()
    }
}
