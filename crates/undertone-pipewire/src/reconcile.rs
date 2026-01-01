//! State reconciliation for self-healing.

use std::sync::Arc;
use tracing::{debug, info, warn};

use undertone_core::channel::ChannelConfig;

use crate::graph::GraphManager;
use crate::node::VirtualSinkProps;

/// Actions that the reconciler determines need to be taken.
#[derive(Debug, Clone)]
pub enum ReconcileAction {
    /// Create a virtual sink node
    CreateSink(VirtualSinkProps),
    /// Create a link between nodes
    CreateLink {
        output_node: String,
        output_port: String,
        input_node: String,
        input_port: String,
    },
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
    /// This compares the current PipeWire graph state against what
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
                actions.push(ReconcileAction::CreateSink(VirtualSinkProps::stereo(
                    name,
                    description,
                )));
            }
        }

        // Check Wave:3 nodes exist
        let wave3_nodes = self.graph.get_wave3_nodes();
        if wave3_nodes.is_empty() {
            actions.push(ReconcileAction::Warn(
                "Wave:3 nodes not found - device may be disconnected".to_string(),
            ));
        }

        // TODO: Check that links between channels and mixes exist
        // TODO: Check that links from mixes to outputs exist

        actions
    }

    /// Check if the graph is in the expected state.
    #[must_use]
    pub fn is_healthy(&self, channels: &[ChannelConfig]) -> bool {
        self.reconcile(channels).is_empty()
    }
}
