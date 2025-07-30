use axum::{extract::State, http::StatusCode};
use axum_distributed_routing::route;
use entities::NetworkStatus;

use crate::{PostgresAppState, response::ApiResponse};

use super::Network;

route!(
    method = GET,
    group = Network,
    path = "/",

    #[axum::debug_handler]
    async fetch_network(state: State<PostgresAppState>) -> ApiResponse<NetworkStatus> {
        ApiResponse::new(match state.fetch_network_status.execute().await {
            Ok(status) => status,
            Err(err) => return err.into(),
        }, StatusCode::OK)
    }
);
