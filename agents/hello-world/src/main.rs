use std::{
    io,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, connect_async, tungstenite::protocol::Message};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;
use uuid::Uuid;

#[derive(Deserialize)]
struct BaseConfig {
    service_id: Uuid,
    token: String,
    helios_base_url: String,
}

#[derive(Deserialize)]
struct Config {
    base: BaseConfig,
}

fn get_config_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    return PathBuf::from("/etc/helios-agent/config.toml");

    #[cfg(target_os = "macos")]
    return PathBuf::from("/Library/Application Support/Helios Agent/config.toml");

    #[cfg(target_os = "windows")]
    {
        let program_data =
            known_folders::get_known_folder_path(known_folders::KnownFolder::ProgramData).unwrap();
        program_data.join("Helios Agent").join("config.toml")
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    panic!("Unsupported operating system");
}

pub struct WebSocketStream {
    inner: tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Stream for WebSocketStream {
    type Item = Result<String, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(Message::Text(message)))) => {
                Poll::Ready(Some(Ok(message.to_string())))
            }
            Poll::Ready(Some(Ok(Message::Binary(_)))) => Poll::Ready(Some(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received binary message",
            )))),
            Poll::Ready(Some(Ok(Message::Ping(_)))) => Poll::Pending,
            Poll::Ready(Some(Ok(Message::Pong(_)))) => Poll::Pending,
            Poll::Ready(Some(Ok(Message::Close(_)))) => Poll::Pending,
            Poll::Ready(Some(Ok(Message::Frame(_)))) => Poll::Ready(Some(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received frame message",
            )))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(io::Error::new(
                io::ErrorKind::Other,
                err.to_string(),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Sink<String> for WebSocketStream {
    type Error = std::io::Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready_unpin(cx)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn start_send(self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        self.get_mut()
            .inner
            .start_send_unpin(Message::Text(item.into()))
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .inner
            .poll_flush_unpin(cx)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .inner
            .poll_close_unpin(cx)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config_path = get_config_path();
    let config_file_content = tokio::fs::read_to_string(config_path).await.unwrap();
    let config = toml::from_str::<Config>(&config_file_content).unwrap();

    let mut url = Url::parse(&config.base.helios_base_url)
        .unwrap()
        .join("/api/v1/agents/websocket")
        .unwrap();

    url.set_scheme("ws").unwrap();

    let (ws_stream, _) = match connect_async(url.as_str()).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    println!("WebSocket handshake completed");

    let mut stream = WebSocketStream { inner: ws_stream };

    protocol::handle_agent(&mut stream, config.base.service_id, &config.base.token)
        .await
        .unwrap();
}
