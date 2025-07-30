use axum::{
    body::Body,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use ports::{api::RouterApiError, repositories::RepositoryError};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub enum ApiResponse<T: Serialize> {
    Ok {
        data: T,
        status_code: StatusCode,
    },
    Error {
        code: &'static str,
        message: String,
        status_code: StatusCode,
    },
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T, status_code: StatusCode) -> Self {
        Self::Ok { data, status_code }
    }

    pub fn error(code: &'static str, message: impl Into<String>, status_code: StatusCode) -> Self {
        Self::Error {
            code,
            message: message.into(),
            status_code,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response<Body> {
        let (success, data, error, status_code) = match self {
            Self::Ok { data, status_code } => (true, Some(json!(data)), None, status_code),
            Self::Error {
                code,
                message,
                status_code,
            } => (
                false,
                None,
                Some(json!({ "code": code, "message": message })),
                status_code,
            ),
        };

        match serde_json::to_vec(&json!({
            "success": success,
            "data": data,
            "error": error
        })) {
            Ok(body) => Response::builder()
                .status(status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(body.into())
                .unwrap(),
            Err(err) => {
                println!("Failed to serialize response: {}", err);
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

impl<T: Serialize> Into<ApiResponse<T>> for RouterApiError {
    fn into(self) -> ApiResponse<T> {
        match self {
            RouterApiError::Unavailable => ApiResponse::error(
                "router-api-unavailable",
                self.to_string(),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            RouterApiError::InvalidResponse(_) => ApiResponse::error(
                "router-api-invalid-response",
                self.to_string(),
                StatusCode::BAD_GATEWAY,
            ),
            RouterApiError::AuthenticationFailed => ApiResponse::error(
                "router-api-authentication-failed",
                self.to_string(),
                StatusCode::BAD_GATEWAY,
            ),
            RouterApiError::Unknown(_) => ApiResponse::error(
                "router-api-unknown-error",
                self.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl<T: Serialize> Into<ApiResponse<T>> for RepositoryError {
    fn into(self) -> ApiResponse<T> {
        match self {
            RepositoryError::NotFound => ApiResponse::error(
                "resource-not-found",
                self.to_string(),
                StatusCode::NOT_FOUND,
            ),
            RepositoryError::CheckViolation => ApiResponse::error(
                "resource-check-violation",
                self.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            RepositoryError::UniqueViolation => ApiResponse::error(
                "resource-unique-violation",
                self.to_string(),
                StatusCode::CONFLICT,
            ),
            RepositoryError::ForeignKeyViolation => ApiResponse::error(
                "resource-foreign-key-violation",
                self.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            RepositoryError::ConnectionFailed => ApiResponse::error(
                "database-connection-failed",
                self.to_string(),
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            RepositoryError::Unknown => ApiResponse::error(
                "database-unknown-error",
                self.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}
