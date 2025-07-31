use axum::{extract::State, http::StatusCode};
use axum_distributed_routing::route;
use domain::{CreateService, CreateServiceError};
use entities::Service;
use tracing::instrument;

use crate::{
    PostgresAppState,
    extractors::ValidJson,
    response::{ApiError, ApiResponse, ApiResult},
    services::Services,
};

impl From<CreateServiceError> for ApiError {
    fn from(err: CreateServiceError) -> Self {
        match err {
            CreateServiceError::DuplicatePortNumber => ApiError::new(
                "duplicate-port-number",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::DuplicatePortType => ApiError::new(
                "duplicate-port-type",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::MissingRequiredPorts => ApiError::new(
                "missing-required-ports",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::InvalidPortConfiguration => ApiError::new(
                "invalid-port-configuration",
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            CreateServiceError::ServiceAlreadyExists => ApiError::new(
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
    body = ValidJson<CreateService>,

    #[instrument(skip(state))]
    async create_service(state: State<PostgresAppState>) -> ApiResult<Service> {
        Ok(state.create_service.execute(body.0).await.map(|service| {
            ApiResponse::new(service, StatusCode::CREATED)
        })?)
    }
);
