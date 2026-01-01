//! Application entry point.

use std::sync::Arc;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use tracing::info;

use crate::bridge::init_ipc;
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

        // Initialize IPC handler before Qt starts
        // This creates the background tokio runtime and connects to the daemon
        let _command_tx = init_ipc(Arc::clone(&self.state));
        info!("IPC handler started");

        // Initialize Qt application
        let mut app = QGuiApplication::new();

        // Create QML engine
        let mut engine = QQmlApplicationEngine::new();

        // Load main QML file from the compiled QRC
        // The path includes the relative directory from build.rs qml_files
        let qml_path = QUrl::from(&QString::from("qrc:/qt/qml/com/undertone/qml/main.qml"));

        if let Some(engine_pin) = engine.as_mut() {
            engine_pin.load(&qml_path);
        }

        info!("QML loaded, starting event loop");

        // Run the Qt event loop
        let exit_code = if let Some(app_pin) = app.as_mut() {
            app_pin.exec()
        } else {
            return Err("Failed to initialize Qt application".into());
        };

        info!(exit_code, "Qt event loop exited");

        if exit_code != 0 {
            return Err(format!("Qt exited with code {exit_code}").into());
        }

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
