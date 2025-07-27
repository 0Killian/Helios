use std::net::IpAddr;

use mac_address::MacAddress;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub mac_address: MacAddress,
    pub last_known_ip: IpAddr,
    pub display_name: String,
    pub is_name_custom: bool,
    pub notes: String,
    pub is_online: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

impl Device {
    /// Updates the device with the provided device informations.
    pub fn update(&mut self, new_device: Device) {
        assert!(self.mac_address == new_device.mac_address);
        self.last_known_ip = new_device.last_known_ip;
        if !self.is_name_custom {
            self.display_name = new_device.display_name;
        }

        self.is_online = new_device.is_online;
        self.last_seen = new_device.last_seen;
    }
}
