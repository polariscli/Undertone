//! IPC client implementation.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{Mutex, mpsc, oneshot};
use tracing::{debug, error, warn};

use crate::error::{IpcError, IpcResult};
use crate::events::Event;
use crate::messages::{Method, Request, Response};

/// IPC client for connecting to the Undertone daemon.
pub struct IpcClient {
    writer: Arc<Mutex<tokio::net::unix::OwnedWriteHalf>>,
    next_id: AtomicU64,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Response>>>>,
    event_rx: mpsc::Receiver<Event>,
}

impl IpcClient {
    /// Connect to the daemon at the given socket path.
    ///
    /// # Errors
    /// Returns an error if the connection fails.
    pub async fn connect(socket_path: &Path) -> IpcResult<Self> {
        let stream = UnixStream::connect(socket_path).await?;
        let (reader, writer) = stream.into_split();

        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Response>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (event_tx, event_rx) = mpsc::channel(64);

        // Spawn reader task
        let pending_clone = Arc::clone(&pending);
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("Connection closed");
                        break;
                    }
                    Ok(_) => {
                        // Try to parse as response first
                        if let Ok(response) = serde_json::from_str::<Response>(&line) {
                            let mut pending = pending_clone.lock().await;
                            if let Some(tx) = pending.remove(&response.id) {
                                let _ = tx.send(response);
                            }
                        }
                        // Try to parse as event
                        else if let Ok(event) = serde_json::from_str::<Event>(&line) {
                            let _ = event_tx.send(event).await;
                        } else {
                            warn!("Unknown message format");
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Read error");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            next_id: AtomicU64::new(1),
            pending,
            event_rx,
        })
    }

    /// Connect to the daemon at the default socket path.
    ///
    /// # Errors
    /// Returns an error if the connection fails.
    pub async fn connect_default() -> IpcResult<Self> {
        Self::connect(&crate::socket_path()).await
    }

    /// Send a request and wait for a response.
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn request(&self, method: Method) -> IpcResult<Response> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = Request { id, method };

        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        let json = serde_json::to_string(&request)? + "\n";

        {
            let mut writer = self.writer.lock().await;
            writer.write_all(json.as_bytes()).await?;
        }

        rx.await.map_err(|_| IpcError::ChannelClosed)
    }

    /// Get the event receiver for incoming events.
    pub fn events(&mut self) -> &mut mpsc::Receiver<Event> {
        &mut self.event_rx
    }
}
