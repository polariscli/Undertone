//! Undertone UI entry point.

use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("undertone=info".parse()?))
        .init();

    // Run the application
    let app = undertone_ui::Application::new();
    app.run()
}
