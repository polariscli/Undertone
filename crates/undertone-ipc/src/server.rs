//! IPC server implementation.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::SinkExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, error, info, warn};

use crate::error::IpcResult;
use crate::events::{Event, EventType};
use crate::messages::{ErrorInfo, Method, Request, Response};

/// IPC server that listens for client connections.
pub struct IpcServer {
    listener: UnixListener,
    clients: Arc<RwLock<HashMap<u64, ClientHandle>>>,
    next_client_id: AtomicU64,
    event_tx: broadcast::Sender<Event>,
    request_tx: mpsc::Sender<(u64, Request, mpsc::Sender<Response>)>,
}

struct ClientHandle {
    subscriptions: Vec<EventType>,
    response_tx: mpsc::Sender<Response>,
}

impl IpcServer {
    /// Create a new IPC server bound to the given socket path.
    ///
    /// # Errors
    /// Returns an error if the socket cannot be created.
    pub async fn bind(
        socket_path: &Path,
    ) -> IpcResult<(Self, mpsc::Receiver<(u64, Request, mpsc::Sender<Response>)>)> {
        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Remove stale socket file if it exists
        if socket_path.exists() {
            tokio::fs::remove_file(socket_path).await?;
        }

        let listener = UnixListener::bind(socket_path)?;
        info!(?socket_path, "IPC server listening");

        let (event_tx, _) = broadcast::channel(256);
        let (request_tx, request_rx) = mpsc::channel(64);

        Ok((
            Self {
                listener,
                clients: Arc::new(RwLock::new(HashMap::new())),
                next_client_id: AtomicU64::new(1),
                event_tx,
                request_tx,
            },
            request_rx,
        ))
    }

    /// Run the server, accepting connections.
    pub async fn run(&self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let client_id = self.next_client_id.fetch_add(1, Ordering::SeqCst);
                    info!(client_id, "Client connected");

                    let clients = Arc::clone(&self.clients);
                    let event_rx = self.event_tx.subscribe();
                    let request_tx = self.request_tx.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            Self::handle_client(client_id, stream, clients, event_rx, request_tx)
                                .await
                        {
                            error!(client_id, error = %e, "Client error");
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "Accept error");
                }
            }
        }
    }

    /// Broadcast an event to all subscribed clients.
    pub fn broadcast(&self, event: Event) {
        let _ = self.event_tx.send(event);
    }

    /// Get a clone of the event sender for broadcasting from other tasks.
    pub fn event_sender(&self) -> broadcast::Sender<Event> {
        self.event_tx.clone()
    }

    async fn handle_client(
        client_id: u64,
        stream: UnixStream,
        clients: Arc<RwLock<HashMap<u64, ClientHandle>>>,
        mut event_rx: broadcast::Receiver<Event>,
        request_tx: mpsc::Sender<(u64, Request, mpsc::Sender<Response>)>,
    ) -> IpcResult<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        let (response_tx, mut response_rx) = mpsc::channel::<Response>(16);

        // Register client
        {
            let mut clients = clients.write().await;
            clients.insert(
                client_id,
                ClientHandle { subscriptions: Vec::new(), response_tx: response_tx.clone() },
            );
        }

        loop {
            tokio::select! {
                // Read request from client
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            debug!(client_id, "Client disconnected");
                            break;
                        }
                        Ok(_) => {
                            if let Ok(request) = serde_json::from_str::<Request>(&line) {
                                debug!(client_id, request_id = request.id, "Received request");
                                let _ = request_tx.send((client_id, request, response_tx.clone())).await;
                            } else {
                                warn!(client_id, "Invalid request format");
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!(client_id, error = %e, "Read error");
                            break;
                        }
                    }
                }

                // Send response to client
                Some(response) = response_rx.recv() => {
                    let json = serde_json::to_string(&response).unwrap() + "\n";
                    if let Err(e) = writer.write_all(json.as_bytes()).await {
                        error!(client_id, error = %e, "Write error");
                        break;
                    }
                }

                // Forward events to client
                Ok(event) = event_rx.recv() => {
                    let clients = clients.read().await;
                    if let Some(handle) = clients.get(&client_id) {
                        if handle.subscriptions.contains(&event.event) || handle.subscriptions.is_empty() {
                            let json = serde_json::to_string(&event).unwrap() + "\n";
                            if let Err(e) = writer.write_all(json.as_bytes()).await {
                                error!(client_id, error = %e, "Event write error");
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Unregister client
        {
            let mut clients = clients.write().await;
            clients.remove(&client_id);
        }

        info!(client_id, "Client handler exiting");
        Ok(())
    }
}
