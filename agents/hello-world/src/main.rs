use std::path::PathBuf;

use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::time::{self, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[derive(Deserialize)]
struct BaseConfig {
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

#[tokio::main]
async fn main() {
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

    let (mut write, mut read) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(t)) => {
                    println!("Received from server: {}", t);
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    });

    let mut interval = time::interval(Duration::from_secs(3));
    let mut count = 0;

    loop {
        interval.tick().await;
        let msg_text = format!("Hello from client! (Message #{})", count);
        println!("Sending to server: {}", msg_text);

        if write.send(Message::Text(msg_text.into())).await.is_err() {
            eprintln!("Failed to send message. Connection closed.");
            break;
        }
        count += 1;
    }
}
