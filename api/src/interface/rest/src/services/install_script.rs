use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use axum_distributed_routing::route;
use domain::{InstallationScript, OperatingSystem};
use serde::Deserialize;
use uuid::Uuid;

use crate::{PostgresAppState, services::Services};

#[derive(Deserialize)]
pub struct InstallScriptQuery {
    os: OperatingSystem,
}

route!(
    method = GET,
    group = Services,
    path = "/{service_id:Uuid}/install-script",
    query = InstallScriptQuery,

    async create_service(state: State<PostgresAppState>) -> impl IntoResponse {
        let InstallationScript { content, file_format, file_name } = state.generate_install_script.execute(query.os, service_id).await;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_str(&file_format).unwrap());
        headers.insert("Content-Disposition", format!("attachment; filename={}", file_name).parse().unwrap());
        (headers, Body::from(content))
    }
);
