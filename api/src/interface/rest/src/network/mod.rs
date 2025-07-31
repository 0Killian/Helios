use axum_distributed_routing::route_group;

use crate::{PostgresAppState, RestV1};

route_group!(Network, PostgresAppState, RestV1, "/network");

mod get;
