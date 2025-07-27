use std::str::FromStr;

use strum::EnumString;
use url::Url;

pub struct Config {
    pub internet_provider: InternetProviderConfig,
    pub database: DatabaseConfig,
    pub scanning: ScanningConfig,
}

pub struct InternetProviderConfig {
    pub kind: InternetProvider,
    pub base_url: Url,
    pub password: String,
}

pub struct DatabaseConfig {
    pub url: Url,
}

pub struct ScanningConfig {
    pub device_scan_delay: u64,
}

#[derive(EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum InternetProvider {
    Bouygues,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        let base_url = Url::parse(
            &std::env::var("INTERNET_PROVIDER_BASE_URL")
                .expect("INTERNET_PROVIDER_BASE_URL must be set"),
        )
        .expect("Invalid URL format for INTERNET_PROVIDER_BASE_URL");
        let password = std::env::var("INTERNET_PROVIDER_PASSWORD")
            .expect("INTERNET_PROVIDER_PASSWORD must be set");
        let kind = InternetProvider::from_str(
            &std::env::var("INTERNET_PROVIDER_KIND").expect("INTERNET_PROVIDER_KIND must be set"),
        )
        .expect("Invalid INTERNET_PROVIDER_KIND");
        let database_url =
            Url::parse(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
                .expect("Invalid URL format for DATABASE_URL");
        let device_scan_delay = std::env::var("DEVICE_SCAN_DELAY_SECONDS")
            .unwrap_or("30".to_string())
            .parse()
            .expect("Invalid DEVICE_SCAN_DELAY_SECONDS");

        Self {
            internet_provider: InternetProviderConfig {
                base_url,
                password,
                kind,
            },
            database: DatabaseConfig { url: database_url },
            scanning: ScanningConfig { device_scan_delay },
        }
    }
}

pub static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(|| Config::from_env());
