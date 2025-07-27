use axum_distributed_routing::route_group;

use crate::{PostgresAppState, RestV1};

route_group!(pub Devices, PostgresAppState, RestV1, "/devices");

pub mod list;
