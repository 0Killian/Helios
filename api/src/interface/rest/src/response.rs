use axum::{
    body::Body,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub enum ApiResponse<T: Serialize> {
    Ok {
        data: T,
        status_code: StatusCode,
        headers: HeaderMap,
    },
    Error {
        code: &'static str,
        message: String,
        status_code: StatusCode,
    },
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T, status_code: StatusCode) -> Self {
        Self::Ok {
            data,
            status_code,
            headers: HeaderMap::new(),
        }
    }

    pub fn with_headers(data: T, status_code: StatusCode, headers: HeaderMap) -> Self {
        Self::Ok {
            data,
            status_code,
            headers,
        }
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
        let (success, data, error, status_code, headers) = match self {
            Self::Ok {
                data,
                status_code,
                headers,
            } => (true, Some(json!(data)), None, status_code, headers),
            Self::Error {
                code,
                message,
                status_code,
            } => (
                false,
                None,
                Some(json!({ "code": code, "message": message })),
                status_code,
                HeaderMap::new(),
            ),
        };

        let mut response = Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json");

        response.headers_mut().unwrap().extend(headers);

        response
            .body(
                serde_json::to_vec(&json!({
                    "success": success,
                    "data": data,
                    "error": error
                }))
                .unwrap()
                .into(),
            )
            .unwrap()
    }
}
