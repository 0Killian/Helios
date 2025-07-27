use axum::{Json, extract::State};
use axum_distributed_routing::{route, route_group};
use entities::NetworkStatus;

use crate::{PostgresAppState, RestV1};

route_group!(Network, PostgresAppState, RestV1, "/network");

route!(
    method = GET,
    group = Network,
    path = "/",

    #[axum::debug_handler]
    async fetch_network(state: State<PostgresAppState>) -> Json<NetworkStatus> {
        Json(state.fetch_network_status.execute().await)
    }
);
