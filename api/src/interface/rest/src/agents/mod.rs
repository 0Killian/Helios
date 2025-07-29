use axum_distributed_routing::route_group;

use crate::{PostgresAppState, RestV1};

route_group!(Agents, PostgresAppState, RestV1, "/agents");

mod websocket;
