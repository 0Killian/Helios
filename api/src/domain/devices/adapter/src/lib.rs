use entities::{Device, Pagination, Service};
use mac_address::MacAddress;

#[async_trait::async_trait]
pub trait DevicesAdapter: Send + Sync {
    /// List all devices stored in the database
    async fn list_devices(&self, pagination: Option<Pagination>) -> Vec<Device>;

    /// List all services attached to a device
    async fn list_services(&self, device_mac: MacAddress) -> Vec<Service>;

    /// Scan the devices connected to the network and persist them in the database. Will repeat the scan every `delay` duration.
    async fn start_devices_scan(&self, delay: chrono::Duration);
}
