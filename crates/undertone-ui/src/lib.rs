//! Undertone UI - Qt6/QML user interface.
//!
//! This crate provides the graphical user interface for Undertone,
//! built with Qt6 and QML via cxx-qt bindings.

pub mod app;
pub mod bridge;
pub mod state;

pub use app::Application;
pub use state::UiState;
