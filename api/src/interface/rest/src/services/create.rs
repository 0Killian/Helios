use axum::{Json, extract::State};
use axum_distributed_routing::route;
use domain::CreateService;
use entities::Service;

use crate::{PostgresAppState, services::Services};

route!(
    method = POST,
    group = Services,
    path = "/",
    body = Json<CreateService>,

    async create_service(state: State<PostgresAppState>) -> Json<Service> {
        Json(state.create_service.execute(body.0).await)
    }
);
