use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub id: String,
    pub msg_type: String,
    pub payload: serde_json::Value,
}

#[derive(Clone)]
pub struct WsServer {
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    broadcaster: broadcast::Sender<String>,
}

impl WsServer {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        WsServer {
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcaster,
        }
    }

    pub async fn broadcast(&self, event: &str, data: serde_json::Value) {
        let msg = serde_json::json!({
            "type": event,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        let _ = self.broadcaster.send(msg.to_string());
    }

    async fn handle_connection(
        stream: TcpStream,
        broadcaster: broadcast::Sender<String>,
        clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    ) {
        let client_id = Uuid::new_v4().to_string();
        println!("🔌 New WebSocket client: {}", client_id);

        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                println!("❌ WebSocket error: {}", e);
                return;
            }
        };

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let mut rx = broadcaster.subscribe();

        // Add client
        {
            let mut clients = clients.lock().await;
            clients.insert(client_id.clone(), broadcaster.clone());
        }

        // Send welcome message
        let welcome = serde_json::json!({
            "type": "connected",
            "data": {
                "client_id": client_id,
                "message": "Connected to RustQL WebSocket! 🦀"
            }
        });
        let _ = ws_sender.send(Message::Text(welcome.to_string().into())).await;

        loop {
            tokio::select! {
                // Receive from client
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            println!("📨 Received: {}", text);
                            let response = serde_json::json!({
                                "type": "message",
                                "data": text.to_string()
                            });
                            let _ = ws_sender.send(
                                Message::Text(response.to_string().into())
                            ).await;
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            println!("👋 Client disconnected: {}", client_id);
                            break;
                        }
                        _ => {}
                    }
                }
                // Broadcast to client
                Ok(msg) = rx.recv() => {
                    let _ = ws_sender.send(Message::Text(msg.into())).await;
                }
            }
        }

        // Remove client
        let mut clients = clients.lock().await;
        clients.remove(&client_id);
    }

    pub async fn start(&self, port: u16) {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await.unwrap();
        println!("🔌 WebSocket server running on ws://{}", addr);

        let broadcaster = self.broadcaster.clone();
        let clients = Arc::clone(&self.clients);

        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let broadcaster = broadcaster.clone();
                let clients = Arc::clone(&clients);
                tokio::spawn(async move {
                    Self::handle_connection(stream, broadcaster, clients).await;
                });
            }
        }
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_server_creation() {
        let server = WsServer::new();
        assert!(server.broadcaster.receiver_count() == 0);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let server = WsServer::new();
        let data = serde_json::json!({"message": "test"});
        server.broadcast("test_event", data).await;
    }
}