use std::{net::IpAddr, str::FromStr};

use config_macro::config;
use strum::EnumString;
use url::Url;

trait FromEnv {
    fn from_env(key: &str, default: Option<&str>) -> Self;
}

impl<T> FromEnv for T
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn from_env(key: &str, default: Option<&str>) -> Self {
        std::env::var(key)
            .ok()
            .as_deref()
            .or(default)
            .expect(&format!("Missing environment variable: {}", key))
            .parse()
            .expect(&format!("Failed to parse environment variable: {}", key))
    }
}

#[config]
pub struct Config {
    #[env("")]
    pub api: ApiConfig,
    #[env("ROUTER_API")]
    pub router_api: RouterApiConfig,
    #[env("DATABASE")]
    pub database: DatabaseConfig,
    #[env("SCANNING")]
    pub scanning: ScanningConfig,
    #[env("AGENT")]
    pub agents: AgentsConfig,
}

#[config]
pub struct ApiConfig {
    #[env("LISTEN_ADDRESS", default = "0.0.0.0")]
    pub listen_address: IpAddr,
    #[env("LISTEN_PORT", default = "3000")]
    pub listen_port: u16,
    #[env("BASE_URL", default = "http://localhost:3000")]
    pub base_url: Url,
}

#[config]
pub struct RouterApiConfig {
    #[env("KIND")]
    pub kind: RouterKind,
    #[env("BASE_URL")]
    pub base_url: Url,
    #[env("PASSWORD")]
    pub password: String,
}

#[config]
pub struct DatabaseConfig {
    #[env("URL")]
    pub url: Url,
}

#[config]
pub struct ScanningConfig {
    #[env("DEVICE_SCAN_DELAY", default = "60")]
    pub device_scan_delay: u64,
}

#[config]
pub struct AgentsConfig {
    #[env("HELLO_WORLD")]
    pub hello_world: BaseAgentConfig,
    #[env("HELLO_WORLD2")]
    pub hello_world2: BaseAgentConfig,
}

#[config]
pub struct BaseAgentConfig {
    #[env("DOWNLOAD_BASE_URL")]
    pub download_base_url: String,
}

#[derive(EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum RouterKind {
    Bbox,
}

pub static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(|| {
    dotenv::dotenv().ok();
    Config::from_env("API", None)
});
