use entities::{Device, WanConnectivity, WanStats};

#[async_trait::async_trait]
pub trait InternetProviderApi: Send + Sync {
    async fn wan_connectivity(&self) -> WanConnectivity;
    async fn list_devices(&self) -> Vec<Device>;
    async fn wan_stats(&self) -> WanStats;
}
