use axum::{Json, extract::State};
use axum_distributed_routing::route;
use entities::Pagination;
use futures::{StreamExt, stream::FuturesUnordered};
use serde::Deserialize;

use crate::{
    Services,
    devices::{DeviceResponse, Devices},
};

#[derive(Debug, Deserialize)]
pub struct ListDeviceQuery {
    #[serde(flatten, deserialize_with = "entities::deserialize_option_pagination")]
    pub pagination: Option<Pagination>,

    #[serde(default)]
    pub full: bool,
}

route!(
    method = GET,
    group = Devices,
    path = "/",
    query = ListDeviceQuery,

    async fetch_devices(state: State<Services>) -> Json<Vec<DeviceResponse>> {
        let list = state.devices.lock().await.list_devices(query.pagination).await;

        if list.is_empty() {
            return Json(vec![]);
        }

        let response = FuturesUnordered::from_iter(list.into_iter().map(async |device| {
            if query.full {
                let mac = device.mac_address;
                DeviceResponse {
                    device,
                    services: Some(state.devices.lock().await.list_services(mac).await),
                }
            } else {
                DeviceResponse {
                    device,
                    services: None,
                }
            }
        }));

        Json(response.collect().await)
    }
);
