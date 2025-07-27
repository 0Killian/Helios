use axum_distributed_routing::route_group;
use entities::{Device, Service};
use serde::Serialize;

use crate::{RestV1, Services};

route_group!(pub Devices, Services, RestV1, "/devices");

pub mod list;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceResponse {
    pub device: Device,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<Service>>,
}
