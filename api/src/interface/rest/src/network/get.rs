use axum::{extract::State, http::StatusCode};
use axum_distributed_routing::route;
use entities::NetworkStatus;
use tracing::instrument;

use crate::{
    PostgresAppState,
    response::{ApiResponse, ApiResult},
};

use super::Network;

route!(
    method = GET,
    group = Network,
    path = "/",

    #[instrument(skip(state))]
    async fetch_network(state: State<PostgresAppState>) -> ApiResult<NetworkStatus> {
        Ok(ApiResponse::new(match state.fetch_network_status.execute().await {
            Ok(status) => status,
            Err(err) => return Err(err.into()),
        }, StatusCode::OK))
    }
);
