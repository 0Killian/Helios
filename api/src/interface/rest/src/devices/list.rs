use axum::{Json, extract::State};
use axum_distributed_routing::route;
use entities::{FullDevice, Pagination};
use serde::Deserialize;

use crate::{PostgresAppState, devices::Devices};

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

    async fetch_devices(state: State<PostgresAppState>) -> Json<Vec<FullDevice>> {
        Json(state.list_devices.execute(query.pagination, query.full).await)
    }
);
