use axum::{Json, extract::State};
use axum_distributed_routing::route;
use serde::Deserialize;

use crate::{
    Services,
    devices::{DeviceResponse, Devices},
};

#[derive(Debug, Deserialize)]
pub struct ListDeviceQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    #[serde(default)]
    pub full: bool,
}

route!(
    method = GET,
    group = Devices,
    path = "/",
    query = ListDeviceQuery,

    async fetch_devices(state: State<Services>) -> Json<Vec<DeviceResponse>> {
        let mut devices = state.devices.lock().await;
        let list = devices.list_devices(query.page, query.limit).await;

        if list.is_empty() {
            return Json(vec![]);
        }

        let mut response = list.iter().map(|device| {
            if query.full {
                DeviceResponse {
                    device,
                    services: devices.list_services(device.device_mac).await,
                }
            } else {
                DeviceResponse {
                    device,
                    services: None,
                }
            }
        }).collect::<Vec<_>>();

        Json(list)
    }
);
