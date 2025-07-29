use axum::{
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use axum_distributed_routing::route;

use crate::agents::Agents;

route!(
    group = Agents,
    method = GET,
    path = "/websocket",

    async websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(handle_websocket)
    }
);

async fn handle_websocket(mut socket: WebSocket) {
    println!("WebSocket connected!");

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("Received text message: {}", t);
                    let response = format!("Echo: {}", t);
                    if socket.send(Message::Text(response.into())).await.is_err() {
                        // Client disconnected
                        break;
                    }
                }
                Message::Binary(_) => {
                    println!("Received binary message (not supported)");
                }
                Message::Ping(_) | Message::Pong(_) => {
                    // The library handles these automatically.
                }
                Message::Close(_) => {
                    println!("Client disconnected");
                    break;
                }
            }
        } else {
            // Client disconnected unexpectedly.
            println!("Client disconnected");
            break;
        }
    }
}
