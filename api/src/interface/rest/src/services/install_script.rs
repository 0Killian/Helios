use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode, header},
    response::IntoResponse,
};
use axum_distributed_routing::route;
use domain::{GenerateInstallScriptError, InstallationScript, OperatingSystem};
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::{PostgresAppState, extractors::ValidQuery, services::Services};

#[derive(Deserialize, Validate, Debug)]
pub struct InstallScriptQuery {
    os: OperatingSystem,
}

route!(
    method = GET,
    group = Services,
    path = "/{service_id:Uuid}/install-script",
    query = ValidQuery<InstallScriptQuery>,

    #[instrument(skip(state, query), fields(os = ?query.os, service_id = %service_id))]
    async create_service(state: State<PostgresAppState>) -> Response<Body> {
        let InstallationScript { content, file_format, file_name } = match state.generate_install_script.execute(query.os, service_id).await {
            Ok(script) => script,
            Err(GenerateInstallScriptError::ServiceNotFound) => return (
                StatusCode::NOT_FOUND,
                GenerateInstallScriptError::ServiceNotFound.to_string()
            ).into_response(),
            Err(GenerateInstallScriptError::DatabaseError(err)) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string()
            ).into_response(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, file_format.parse().unwrap());
        headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename={}", file_name).parse().unwrap());
        (headers, Body::from(content)).into_response()
    }
);
