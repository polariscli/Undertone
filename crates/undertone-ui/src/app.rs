//! Application entry point.

use std::sync::Arc;
use tracing::info;

use crate::state::UiState;

/// Main application struct.
pub struct Application {
    state: Arc<UiState>,
}

impl Application {
    /// Create a new application.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(UiState::new()),
        }
    }

    /// Run the application.
    ///
    /// This initializes Qt and runs the main event loop.
    ///
    /// # Errors
    /// Returns an error if the application fails to start.
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Undertone UI");

        // TODO: Initialize Qt application via cxx-qt
        // This requires the Qt development packages to be installed
        // and proper cxx-qt setup with build.rs

        // For now, just print a message
        println!("Undertone UI");
        println!("Qt integration pending - install qt6-qtbase-devel and qt6-qtdeclarative-devel");

        Ok(())
    }

    /// Get the UI state.
    #[must_use]
    pub fn state(&self) -> Arc<UiState> {
        Arc::clone(&self.state)
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
