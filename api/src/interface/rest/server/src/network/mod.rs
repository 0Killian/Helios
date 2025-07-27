use axum::{Json, extract::State};
use axum_distributed_routing::{route, route_group};
use entities::{WanConnectivity, WanStats};
use serde::Serialize;

use crate::{RestV1, Services};

route_group!(Network, Services, RestV1, "/network");

#[derive(Debug, Serialize)]
pub struct NetworkResponse {
    pub connectivity: WanConnectivity,
    pub stats: WanStats,
}

route!(
    method = GET,
    group = Network,
    path = "/",

    #[axum::debug_handler]
    async fetch_network(state: State<Services>) -> Json<NetworkResponse> {
        let stats = state.infrastructure.internet_provider_api.wan_stats().await;
        let connectivity = state.infrastructure.internet_provider_api.wan_connectivity().await;

        Json(NetworkResponse { connectivity, stats })
    }
);
