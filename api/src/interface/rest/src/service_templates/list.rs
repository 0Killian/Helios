use axum::{extract::State, http::StatusCode};
use axum_distributed_routing::route;
use entities::ServiceTemplate;
use tracing::instrument;

use crate::{PostgresAppState, response::ApiResponse, service_templates::ServiceTemplates};

route!(
    method = GET,
    path = "/",
    group = ServiceTemplates,

    #[instrument(skip(state))]
    async list_service_templates(state: State<PostgresAppState>) -> ApiResponse<Vec<ServiceTemplate>> {
        ApiResponse::new(state.list_service_templates.execute().await, StatusCode::OK)
    }
);
