use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::Serialize;

mod mac_address;

pub use mac_address::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WanStats {
    pub download: WanStatsItem,
    pub upload: WanStatsItem,
    pub active_sessions: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WanStatsItem {
    pub max_bandwidth: usize,           // in kbps
    pub current_bandwidth: usize,       // in kbps
    pub total_since_last_reboot: usize, // in bytes
    pub packets_lost: usize,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WanConnectivity {
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
    pub gateway: IpAddr,
    pub status: WanStatus,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub uptime: chrono::Duration,
}

#[derive(Debug, Serialize)]
pub enum WanStatus {
    Up,
    Down,
}
