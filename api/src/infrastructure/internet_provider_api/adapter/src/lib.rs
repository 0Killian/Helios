use entities::{Device, WanConnectivity, WanStats};

#[async_trait::async_trait]
pub trait InternetProviderApiAdapter: Send + Sync {
    async fn wan_connectivity(&mut self) -> WanConnectivity;
    async fn list_devices(&mut self) -> Vec<Device>;
    async fn wan_stats(&mut self) -> WanStats;
}
