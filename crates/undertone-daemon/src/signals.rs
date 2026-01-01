//! Signal handling for graceful shutdown.

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

/// Set up signal handlers for graceful shutdown.
///
/// Returns a receiver that will receive a message when a shutdown
/// signal (SIGTERM, SIGINT) is received.
pub fn setup_signal_handlers() -> Result<mpsc::Receiver<()>> {
    let (tx, rx) = mpsc::channel(1);

    // Handle SIGTERM
    let tx_term = tx.clone();
    tokio::spawn(async move {
        if let Ok(mut stream) = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        ) {
            stream.recv().await;
            info!("Received SIGTERM");
            let _ = tx_term.send(()).await;
        }
    });

    // Handle SIGINT (Ctrl+C)
    let tx_int = tx;
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received SIGINT");
            let _ = tx_int.send(()).await;
        }
    });

    Ok(rx)
}
