use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use axum_distributed_routing::route;
use domain::{CreateService, CreateServiceError};
use entities::Service;

use crate::{PostgresAppState, response::ApiResponse, services::Services};

impl From<CreateServiceError> for ApiResponse<Service> {
    fn from(err: CreateServiceError) -> Self {
        match err {
            CreateServiceError::DuplicatePortNumber => ApiResponse::error(
                "duplicate-port-number",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::DuplicatePortType => ApiResponse::error(
                "duplicate-port-type",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::MissingRequiredPorts => ApiResponse::error(
                "missing-required-ports",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::InvalidPortConfiguration => ApiResponse::error(
                "invalid-port-configuration",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::ServiceAlreadyExists => ApiResponse::error(
                "service-already-exists",
                err.to_string(),
                StatusCode::CONFLICT,
            ),
            CreateServiceError::DatabaseError(err) => err.into(),
        }
    }
}

route!(
    method = POST,
    group = Services,
    path = "/",
    body = Json<CreateService>,

    async create_service(state: State<PostgresAppState>) -> ApiResponse<Service> {
        match state.create_service.execute(body.0).await {
            Ok(service) => ApiResponse::new(service, StatusCode::CREATED),
            Err(err) => err.into(),
        }
    }
);
