//! Undertone PipeWire - Graph management and node control.
//!
//! This crate handles all interactions with PipeWire, including:
//! - Connecting to the PipeWire daemon
//! - Creating and managing virtual audio nodes
//! - Managing links between nodes
//! - Monitoring the audio graph for changes

pub mod error;
pub mod factory;
pub mod graph;
pub mod link;
pub mod monitor;
pub mod node;
pub mod reconcile;
pub mod runtime;

pub use error::{PwError, PwResult};
pub use factory::{FactoryRequest, FactoryResponse, NodeFactory};
pub use graph::GraphManager;
pub use monitor::{GraphEvent, GraphMonitor};
pub use runtime::PipeWireRuntime;
