use std::str::FromStr;

use entities::InternetProvider;
use url::Url;

pub struct Config {
    pub internet_provider: InternetProviderConfig,
}

pub struct InternetProviderConfig {
    pub kind: InternetProvider,
    pub base_url: Url,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        let base_url = std::env::var("INTERNET_PROVIDER_BASE_URL")
            .expect("INTERNET_PROVIDER_BASE_URL must be set");
        let password = std::env::var("INTERNET_PROVIDER_PASSWORD")
            .expect("INTERNET_PROVIDER_PASSWORD must be set");
        let kind = InternetProvider::from_str(
            &std::env::var("INTERNET_PROVIDER_KIND").expect("INTERNET_PROVIDER_KIND must be set"),
        )
        .expect("Invalid INTERNET_PROVIDER_KIND");

        Self {
            internet_provider: InternetProviderConfig {
                base_url: Url::parse(&base_url)
                    .expect("Invalid URL format for INTERNET_PROVIDER_BASE_URL"),
                password,
                kind,
            },
        }
    }
}

pub static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(|| Config::from_env());
