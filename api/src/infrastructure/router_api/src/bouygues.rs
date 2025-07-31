use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use common::hashmap;
use entities::{Device, WanConnectivity, WanStats, WanStatsItem, WanStatus};
use ports::api::{RouterApi, RouterApiError, RouterApiResult};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, de::Visitor};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, instrument, warn};

#[derive(Error, Debug)]
enum BboxRouterApiError {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API returned unexpected status: {0}")]
    UnexpectedStatus(StatusCode),

    #[error("Failed to parse response body: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl From<BboxRouterApiError> for RouterApiError {
    fn from(err: BboxRouterApiError) -> Self {
        // TODO: Log the error
        match err {
            BboxRouterApiError::RequestFailed(e) => {
                error!("An error occurred while making the request: {}", e);
                RouterApiError::Unavailable
            }
            BboxRouterApiError::UnexpectedStatus(status) if status == StatusCode::UNAUTHORIZED => {
                error!("Failed to authenticate with the router");
                RouterApiError::AuthenticationFailed
            }
            BboxRouterApiError::UnexpectedStatus(status) => {
                error!("API returned unexpected status: {}", status);
                RouterApiError::InvalidResponse(err.to_string())
            }
            BboxRouterApiError::ParseError(ref e) => {
                error!("Failed to parse response body: {}", e);
                RouterApiError::InvalidResponse(err.to_string())
            }
            BboxRouterApiError::MissingField(ref e) => {
                error!("Missing required field in response: {}", e);
                RouterApiError::InvalidResponse(err.to_string())
            }
        }
    }
}

pub struct BboxRouterApi {
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

impl BboxRouterApi {
    pub async fn new(base_url: Url, password: String) -> Result<Self, RouterApiError> {
        let api = Self {
            client: reqwest::Client::new(),
            base_url,
            password,
            cookie: RwLock::new(String::new()),
        };

        api.authenticate().await?;

        Ok(api)
    }

    #[instrument(skip(self))]
    async fn authenticate(&self) -> Result<(), BboxRouterApiError> {
        let response = self
            .client
            .post(self.base_url.clone().join("/api/v1/login").unwrap())
            .form(&hashmap! {
                "password" => self.password.clone(),
                "remember" => 0.to_string()
            })
            .send()
            .await
            .map_err(BboxRouterApiError::from)?;

        if response.status() != StatusCode::OK {
            return Err(BboxRouterApiError::UnexpectedStatus(response.status()));
        }

        *self.cookie.write().await = response
            .headers()
            .get("Set-Cookie")
            .ok_or(BboxRouterApiError::MissingField("Set-Cookie".to_string()))?
            .to_str()
            .map_err(|_| BboxRouterApiError::MissingField("auth-cookie".to_string()))?
            .to_string();

        Ok(())
    }

    async fn handle_disconnect<F>(
        &self,
        callback: F,
    ) -> Result<reqwest::Response, BboxRouterApiError>
    where
        F: AsyncFn(&Self) -> reqwest::RequestBuilder,
    {
        let response = callback(self).await.send().await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            self.authenticate().await?;
            Ok(callback(self).await.send().await?)
        } else {
            Ok(response)
        }
    }
}

#[async_trait::async_trait]
impl RouterApi for BboxRouterApi {
    #[instrument(skip(self))]
    async fn wan_connectivity(&self) -> RouterApiResult<WanConnectivity> {
        #[derive(Deserialize)]
        struct WanOuter {
            wan: BboxWanResponse,
        }

        #[derive(Deserialize)]
        struct InfoOuter {
            device: BboxInfo,
        }

        let wan = &self
            .client
            .get(self.base_url.clone().join("/api/v1/wan/ip").unwrap())
            .send()
            .await
            .map_err(BboxRouterApiError::from)?
            .json::<Vec<WanOuter>>()
            .await
            .map_err(BboxRouterApiError::from)?[0]
            .wan;

        let info = &self
            .client
            .get(self.base_url.clone().join("/api/v1/device").unwrap())
            .send()
            .await
            .map_err(BboxRouterApiError::from)?
            .json::<Vec<InfoOuter>>()
            .await
            .map_err(BboxRouterApiError::from)?[0]
            .device;

        Ok(WanConnectivity {
            ipv4: wan.ip.address,
            ipv6: wan.ip.ip6address.first().map(|ip| ip.ipaddress),
            gateway: IpAddr::V4(wan.ip.gateway),
            status: WanStatus::Up, // TODO: Implement status detection
            uptime: chrono::Duration::seconds(info.uptime.value.unwrap_or(0) as i64),
        })
    }

    #[instrument(skip(self))]
    async fn list_devices(&self) -> RouterApiResult<Vec<Device>> {
        #[derive(Deserialize)]
        struct HostsResponse {
            hosts: Hosts,
        }

        #[derive(Deserialize)]
        struct Hosts {
            list: Vec<BboxDevice>,
        }

        let devices = &self
            .handle_disconnect(async |self| {
                self.client
                    .get(self.base_url.clone().join("/api/v1/hosts").unwrap())
                    .header("Cookie", self.cookie.read().await.clone())
            })
            .await?
            .json::<Vec<HostsResponse>>()
            .await
            .map_err(BboxRouterApiError::from)?[0]
            .hosts
            .list;

        if devices.is_empty() {
            warn!("Router API returned an empty list of devices. This should never happen!");
            return Ok(Vec::new());
        }

        devices
            .iter()
            .map(|device| {
                Ok(Device {
                    last_seen: chrono::Utc::now()
                        - chrono::Duration::seconds(
                            device
                                .lastseen
                                .value
                                .ok_or(BboxRouterApiError::MissingField(
                                    "device.lastSeen".to_string(),
                                ))
                                .map_err(BboxRouterApiError::from)?
                                as i64,
                        ),
                    mac_address: device.macaddress.parse().unwrap(),
                    last_known_ip: device.ipaddress,
                    display_name: device.hostname.clone(),
                    is_name_custom: false,
                    notes: String::new(),
                    is_online: device.active == 1,
                    last_scanned: chrono::Utc::now(),
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    #[instrument(skip(self))]
    async fn wan_stats(&self) -> RouterApiResult<WanStats> {
        #[derive(Deserialize)]
        struct Response {
            wan: WanResponse,
        }

        #[derive(Deserialize)]
        struct WanResponse {
            ip: IpResponse,
        }

        #[derive(Deserialize)]
        struct IpResponse {
            stats: BboxWanStats,
        }

        let stats = &self
            .handle_disconnect(async |self| {
                self.client
                    .get(self.base_url.clone().join("/api/v1/wan/ip/stats").unwrap())
                    .header("Cookie", self.cookie.read().await.clone())
            })
            .await?
            .json::<Vec<Response>>()
            .await
            .map_err(BboxRouterApiError::from)?[0]
            .wan
            .ip
            .stats;

        let sessions = &self
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
            .await?
            .json::<Vec<BboxWanSessionsSummary>>()
            .await
            .map_err(BboxRouterApiError::from)?[0];

        Ok(WanStats {
            download: WanStatsItem {
                max_bandwidth: stats.rx.max_bandwidth.value.ok_or(
                    BboxRouterApiError::MissingField("download.max_bandwidth".to_string()),
                )? as usize,
                current_bandwidth: stats.rx.bandwidth.value.ok_or(
                    BboxRouterApiError::MissingField("download.current_bandwidth".to_string()),
                )? as usize,
                packets_lost: stats.rx.packetsdiscards.value.ok_or(
                    BboxRouterApiError::MissingField("download.packets_lost".to_string()),
                )? as usize
                    + stats
                        .rx
                        .packetserrors
                        .value
                        .ok_or(BboxRouterApiError::MissingField(
                            "download.packets_errors".to_string(),
                        ))? as usize,
                total_since_last_reboot: stats.rx.bytes.value.ok_or(
                    BboxRouterApiError::MissingField(
                        "download.total_since_last_reboot".to_string(),
                    ),
                )? as usize,
            },
            upload: WanStatsItem {
                max_bandwidth: stats.tx.max_bandwidth.value.ok_or(
                    BboxRouterApiError::MissingField("upload.max_bandwidth".to_string()),
                )? as usize,
                current_bandwidth: stats.tx.bandwidth.value.ok_or(
                    BboxRouterApiError::MissingField("upload.current_bandwidth".to_string()),
                )? as usize,
                packets_lost: stats.tx.packetsdiscards.value.ok_or(
                    BboxRouterApiError::MissingField("upload.packets_lost".to_string()),
                )? as usize
                    + stats
                        .tx
                        .packetserrors
                        .value
                        .ok_or(BboxRouterApiError::MissingField(
                            "upload.packets_errors".to_string(),
                        ))? as usize,
                total_since_last_reboot: stats.tx.bytes.value.ok_or(
                    BboxRouterApiError::MissingField("upload.total_since_last_reboot".to_string()),
                )? as usize,
            },
            active_sessions: sessions
                .currentip
                .value
                .ok_or(BboxRouterApiError::MissingField(
                    "active_sessions".to_string(),
                ))? as usize,
        })
    }
}
