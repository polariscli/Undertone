//! Undertone Core - Business logic for channels, routing, and state management.
//!
//! This crate contains the core domain models and business logic that is shared
//! between the daemon and other components.

pub mod channel;
pub mod command;
pub mod error;
pub mod mixer;
pub mod profile;
pub mod routing;
pub mod state;

pub use channel::{Channel, ChannelConfig, ChannelState};
pub use command::Command;
pub use error::{Error, Result};
pub use mixer::{MixType, MixerState};
pub use profile::Profile;
pub use routing::{AppRoute, RouteRule};
pub use state::{DaemonEvent, DaemonState};
