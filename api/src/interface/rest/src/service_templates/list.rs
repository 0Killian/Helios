use axum::{Json, extract::State};
use axum_distributed_routing::route;
use entities::ServiceTemplate;

use crate::{PostgresAppState, service_templates::ServiceTemplates};

route!(
    method = GET,
    path = "/",
    group = ServiceTemplates,

    async list_service_templates(state: State<PostgresAppState>) -> Json<Vec<ServiceTemplate>> {
        Json(state.list_service_templates.execute().await)
    }
);
