//! Undertone UI entry point.

use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("undertone=info".parse()?))
        .init();

    // Use KDE desktop style for proper Kirigami integration
    // This requires kf6-qqc2-desktop-style package
    // SAFETY: Setting env var before Qt initialization, no other threads running
    unsafe {
        std::env::set_var("QT_QUICK_CONTROLS_STYLE", "org.kde.desktop");
    }

    // Run the application
    let app = undertone_ui::Application::new();
    app.run()
}
