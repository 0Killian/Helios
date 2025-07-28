use axum_distributed_routing::route_group;

use crate::{PostgresAppState, RestV1};

mod list;

route_group!(
    ServiceTemplates,
    PostgresAppState,
    RestV1,
    "/service-templates"
);
