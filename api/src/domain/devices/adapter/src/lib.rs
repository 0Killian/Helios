use entities::{Device, Pagination, Service};
use mac_address::MacAddress;

#[async_trait::async_trait]
pub trait DevicesAdapter: Send {
    /// List all devices stored in the database
    async fn list_devices(&self, pagination: Option<Pagination>) -> Vec<Device>;

    /// List all services attached to a device
    async fn list_services(&self, device_mac: MacAddress) -> Vec<Service>;

    /// Scan the devices connected to the network and persist them in the database
    async fn scan_devices(&self);
}
