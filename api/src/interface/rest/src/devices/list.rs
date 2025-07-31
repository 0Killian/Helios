use axum::{extract::State, http::StatusCode};
use axum_distributed_routing::route;
use entities::{FullDevice, Pagination};
use serde::Deserialize;
use tracing::instrument;
use validator::Validate;

use crate::{
    PostgresAppState,
    devices::Devices,
    extractors::ValidQuery,
    response::{ApiResponse, ApiResult},
};

#[derive(Debug, Deserialize, Validate)]
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
    query = ValidQuery<ListDeviceQuery>,

    #[instrument(skip(state), fields(
        full = %query.full,
        pagination.page = query.pagination.map(|p| p.page),
        pagination.limit = query.pagination.map(|p| p.limit),
    ))]
    async fetch_devices(state: State<PostgresAppState>) -> ApiResult<Vec<FullDevice>> {
        Ok(ApiResponse::new(match state.list_devices.execute(query.pagination, query.full).await {
            Ok(devices) => devices,
            Err(err) => return Err(err.into()),
        }, StatusCode::OK))
    }
);
