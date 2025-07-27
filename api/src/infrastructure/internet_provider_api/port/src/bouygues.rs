use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use common::hashmap;
use entities::{Device, WanConnectivity, WanStats, WanStatsItem, WanStatus};
use ip_api_adapter::InternetProviderApiAdapter;
use reqwest::{StatusCode, Url};
use serde::{Deserialize, de::Visitor};
use tokio::sync::RwLock;

pub struct BboxInternetProviderApiPort {
    client: reqwest::Client,
    base_url: Url,
    password: String,
    cookie: RwLock<String>,
}

#[derive(Debug)]
struct Integer {
    value: Option<isize>,
}

impl<'de> Deserialize<'de> for Integer {
    fn deserialize<D>(deserializer: D) -> Result<Integer, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Either a string or an integer
        struct IntegerVisitor;

        impl<'de> Visitor<'de> for IntegerVisitor {
            type Value = Integer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("integer or string")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Integer, E> {
                Ok(Integer {
                    value: Some(value as isize),
                })
            }

            fn visit_u64<E>(self, value: u64) -> Result<Integer, E> {
                Ok(Integer {
                    value: Some(value as isize),
                })
            }

            fn visit_str<E>(self, value: &str) -> Result<Integer, E> {
                Ok(Integer {
                    value: value.parse().ok(),
                })
            }
        }

        deserializer
            .deserialize_any(IntegerVisitor)
            .and_then(|integer: Integer| {
                Ok(Integer {
                    value: integer.value,
                })
            })
    }
}

#[derive(Deserialize, Debug)]
struct BboxDevice {
    hostname: String,
    macaddress: String,
    ipaddress: IpAddr,
    lastseen: Integer,
    active: usize,
}

#[derive(Deserialize)]
struct BboxWanIpv6Address {
    ipaddress: Ipv6Addr,
}

#[derive(Deserialize)]
struct BboxWanIpResponse {
    address: Ipv4Addr,
    gateway: Ipv4Addr,
    ip6address: Vec<BboxWanIpv6Address>,
}

#[derive(Deserialize)]
struct BboxWanResponse {
    ip: BboxWanIpResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BboxWanStatsItem {
    bytes: Integer,
    packetserrors: Integer,
    packetsdiscards: Integer,
    bandwidth: Integer,
    max_bandwidth: Integer,
}

#[derive(Deserialize)]
struct BboxWanStats {
    rx: BboxWanStatsItem,
    tx: BboxWanStatsItem,
}

#[derive(Deserialize)]
struct BboxWanSessionsSummary {
    currentip: Integer,
}

#[derive(Debug, Deserialize)]
pub struct BboxInfo {
    uptime: Integer,
}

impl BboxInternetProviderApiPort {
    pub async fn new(base_url: Url, password: String) -> BboxInternetProviderApiPort {
        let api = BboxInternetProviderApiPort {
            client: reqwest::Client::new(),
            base_url,
            password,
            cookie: RwLock::new(String::new()),
        };

        api.authenticate().await;

        api
    }

    async fn authenticate(&self) {
        let response = self
            .client
            .post(self.base_url.clone().join("/api/v1/login").unwrap())
            .form(&hashmap! {
                "password" => self.password.clone(),
                "remember" => 0.to_string()
            })
            .send()
            .await
            .unwrap();

        *self.cookie.write().await = response
            .headers()
            .get("Set-Cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    }

    async fn handle_disconnect<F>(&self, callback: F) -> reqwest::Response
    where
        F: AsyncFn(&Self) -> reqwest::RequestBuilder,
    {
        let response = callback(self).await.send().await.unwrap();

        if response.status() == StatusCode::UNAUTHORIZED {
            self.authenticate().await;
            callback(self).await.send().await.unwrap()
        } else {
            response
        }
    }
}

#[async_trait::async_trait]
impl InternetProviderApiAdapter for BboxInternetProviderApiPort {
    async fn wan_connectivity(&self) -> WanConnectivity {
        let response: Vec<serde_json::Value> = self
            .client
            .get(self.base_url.clone().join("/api/v1/wan/ip").unwrap())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let wan: BboxWanResponse =
            serde_json::from_value(response[0].as_object().unwrap()["wan"].clone()).unwrap();

        assert!(wan.ip.ip6address.len() > 0);

        let response: Vec<serde_json::Value> = self
            .client
            .get(self.base_url.clone().join("/api/v1/device").unwrap())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let info: BboxInfo =
            serde_json::from_value(response[0].as_object().unwrap()["device"].clone()).unwrap();

        WanConnectivity {
            ipv4: wan.ip.address,
            ipv6: wan.ip.ip6address[0].ipaddress,
            gateway: IpAddr::V4(wan.ip.gateway),
            status: WanStatus::Up, // TODO: Implement status detection
            uptime: chrono::Duration::seconds(info.uptime.value.unwrap_or(0) as i64),
        }
    }

    async fn list_devices(&self) -> Vec<Device> {
        let response: Vec<serde_json::Value> = self
            .handle_disconnect(async |self| {
                self.client
                    .get(self.base_url.clone().join("/api/v1/hosts").unwrap())
                    .header("Cookie", self.cookie.read().await.clone())
            })
            .await
            .json()
            .await
            .unwrap();

        let devices: Vec<BboxDevice> =
            serde_json::from_value(response[0].as_object().unwrap()["hosts"]["list"].clone())
                .unwrap();

        devices
            .iter()
            .map(|device| Device {
                last_seen: chrono::Utc::now()
                    - chrono::Duration::seconds(device.lastseen.value.unwrap() as i64),
                mac_address: device.macaddress.parse().unwrap(),
                last_known_ip: device.ipaddress,
                display_name: device.hostname.clone(),
                is_name_custom: false,
                notes: String::new(),
                is_online: device.active == 1,
                last_scanned: chrono::Utc::now(),
            })
            .collect()
    }

    async fn wan_stats(&self) -> WanStats {
        let stats: Vec<serde_json::Value> = self
            .handle_disconnect(async |self| {
                self.client
                    .get(self.base_url.clone().join("/api/v1/wan/ip/stats").unwrap())
                    .header("Cookie", self.cookie.read().await.clone())
            })
            .await
            .json()
            .await
            .unwrap();

        let stats: BboxWanStats =
            serde_json::from_value(stats[0]["wan"]["ip"]["stats"].clone()).unwrap();

        let sessions: Vec<serde_json::Value> = self
            .handle_disconnect(async |self| {
                self.client
                    .get(
                        self.base_url
                            .clone()
                            .join("/api/v1/wan/diags/sessions")
                            .unwrap(),
                    )
                    .header("Cookie", self.cookie.read().await.clone())
            })
            .await
            .json()
            .await
            .unwrap();

        let sessions: BboxWanSessionsSummary = serde_json::from_value(sessions[0].clone()).unwrap();

        WanStats {
            download: WanStatsItem {
                max_bandwidth: stats.rx.max_bandwidth.value.unwrap() as usize,
                current_bandwidth: stats.rx.bandwidth.value.unwrap() as usize,
                packets_lost: stats.rx.packetsdiscards.value.unwrap() as usize
                    + stats.rx.packetserrors.value.unwrap() as usize,
                total_since_last_reboot: stats.rx.bytes.value.unwrap() as usize,
            },
            upload: WanStatsItem {
                max_bandwidth: stats.tx.max_bandwidth.value.unwrap() as usize,
                current_bandwidth: stats.tx.bandwidth.value.unwrap() as usize,
                packets_lost: stats.tx.packetsdiscards.value.unwrap() as usize
                    + stats.tx.packetserrors.value.unwrap() as usize,
                total_since_last_reboot: stats.tx.bytes.value.unwrap() as usize,
            },
            active_sessions: sessions.currentip.value.unwrap() as usize,
        }
    }
}
