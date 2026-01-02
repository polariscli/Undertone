//! Undertone UI entry point.

use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("undertone=info".parse()?))
        .init();

    // Set KDE platform theme for full integration (native dialogs, icons, etc.)
    // SAFETY: Setting env var before Qt initialization, no other threads running
    if std::env::var("QT_QPA_PLATFORMTHEME").is_err() {
        unsafe {
            std::env::set_var("QT_QPA_PLATFORMTHEME", "kde");
        }
    }

    // Run the application (style is set programmatically in app.rs)
    let app = undertone_ui::Application::new();
    app.run()
}
