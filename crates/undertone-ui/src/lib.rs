//! Undertone UI - Qt6/QML user interface.
//!
//! This crate provides the graphical user interface for Undertone,
//! built with Qt6 and QML via cxx-qt bindings.

pub mod app;
pub mod bridge;
pub mod ipc_handler;
pub mod state;

pub use app::Application;
pub use bridge::init_ipc;
pub use ipc_handler::{IpcHandle, IpcUpdate};
pub use state::UiState;
