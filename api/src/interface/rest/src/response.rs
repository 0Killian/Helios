use axum::{
    body::Body,
    extract::rejection::{JsonRejection, QueryRejection},
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use ports::{api::RouterApiError, repositories::RepositoryError};
use serde::Serialize;
use serde_json::json;
use tracing::error;
use validator::ValidationErrors;

pub type ApiResult<T> = Result<ApiResponse<T>, ApiError>;

#[derive(Debug)]
pub struct ApiResponse<T: Serialize> {
    data: T,
    status_code: StatusCode,
}

#[derive(Debug)]
pub struct ApiError {
    code: &'static str,
    message: String,
    status_code: StatusCode,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T, status_code: StatusCode) -> Self {
        Self { data, status_code }
    }
}

impl ApiError {
    pub fn new(code: &'static str, message: impl Into<String>, status_code: StatusCode) -> Self {
        Self {
            code,
            message: message.into(),
            status_code,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response<Body> {
        match serde_json::to_vec(&json!({
            "success": true,
            "data": Some(json!(self.data)),
        })) {
            Ok(body) => Response::builder()
                .status(self.status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(body.into())
                .unwrap(),
            Err(err) => {
                error!("Failed to serialize response: {}", err);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(
                        json!({
                            "success": false,
                            "error": {
                                "code": "serialization-error",
                                "message": "Failed to format the server response."
                            }
                        })
                        .to_string()
                        .into(),
                    )
                    .unwrap()
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        match serde_json::to_vec(&json!({
            "success": false,
            "error": Some(json!({ "code": self.code, "message": self.message })),
        })) {
            Ok(body) => Response::builder()
                .status(self.status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(body.into())
                .unwrap(),
            Err(err) => {
                error!("Failed to serialize response: {}", err);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(
                        json!({
                            "success": false,
                            "error": {
                                "code": "serialization-error",
                                "message": "Failed to format the server response."
                            }
                        })
                        .to_string()
                        .into(),
                    )
                    .unwrap()
            }
        }
    }
}

impl Into<ApiError> for RouterApiError {
    fn into(self) -> ApiError {
        match self {
            RouterApiError::Unavailable => ApiError::new(
                "router-api-unavailable",
                self.to_string(),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            RouterApiError::InvalidResponse(_) => ApiError::new(
                "router-api-invalid-response",
                self.to_string(),
                StatusCode::BAD_GATEWAY,
            ),
            RouterApiError::AuthenticationFailed => ApiError::new(
                "router-api-authentication-failed",
                self.to_string(),
                StatusCode::BAD_GATEWAY,
            ),
            RouterApiError::Unknown(_) => ApiError::new(
                "router-api-unknown-error",
                self.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl Into<ApiError> for RepositoryError {
    fn into(self) -> ApiError {
        match self {
            RepositoryError::NotFound => ApiError::new(
                "resource-not-found",
                self.to_string(),
                StatusCode::NOT_FOUND,
            ),
            RepositoryError::CheckViolation => ApiError::new(
                "resource-check-violation",
                self.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            RepositoryError::UniqueViolation => ApiError::new(
                "resource-unique-violation",
                self.to_string(),
                StatusCode::CONFLICT,
            ),
            RepositoryError::ForeignKeyViolation => ApiError::new(
                "resource-foreign-key-violation",
                self.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            RepositoryError::ConnectionFailed => ApiError::new(
                "database-connection-failed",
                self.to_string(),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            RepositoryError::Unknown => ApiError::new(
                "database-unknown-error",
                self.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let (status, message) = match rejection {
            JsonRejection::JsonDataError(e) => {
                (StatusCode::BAD_REQUEST, format!("Invalid JSON data: {}", e))
            }
            JsonRejection::JsonSyntaxError(e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid JSON syntax: {}", e),
            ),
            JsonRejection::MissingJsonContentType(_) => (
                StatusCode::BAD_REQUEST,
                "Missing `Content-Type: application/json` header.".to_string(),
            ),
            _ => (
                StatusCode::BAD_REQUEST,
                format!("An unknown error occurred with the JSON payload."),
            ),
        };

        ApiError::new("invalid-json", message, status)
    }
}

impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
        ApiError::new(
            "invalid-query-params",
            format!("Invalid query parameters: {}", rejection),
            StatusCode::BAD_REQUEST,
        )
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(error: ValidationErrors) -> Self {
        ApiError::new(
            "payload-validation-failed",
            format!("Invalid payload: {}", error),
            StatusCode::BAD_REQUEST,
        )
    }
}
