use std::net::IpAddr;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub ip: IpAddr,
    pub mac: String,
    pub connected: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}
