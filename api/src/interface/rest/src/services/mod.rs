use axum_distributed_routing::route_group;

use crate::{PostgresAppState, RestV1};

route_group!(Services, PostgresAppState, RestV1, "/services");

mod create;
mod install_script;
