use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_websocket_connection(
    websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    notification_tx: Arc<broadcast::Sender<String>>,
) {
    let (mut ws_sender, mut ws_receiver) = websocket.split();
    let mut notification_rx = notification_tx.subscribe();
    loop {
        tokio::select! {
            Some(msg) = ws_receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received message: {}", text);
                        // Handle incoming messages if needed
                    }
                    Ok(Message::Close(_)) => break,
                    _ => {}
                }
            }
            Ok(notification) = notification_rx.recv() => {
                if let Err(e) = ws_sender.send(Message::Text(serde_json::to_string(&notification).unwrap())).await {
                    eprintln!("Error sending WebSocket message: {:?}", e);
                    break;
                }
            }
        }
    }
}
