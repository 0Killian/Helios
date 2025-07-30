use entities::{Device, WanConnectivity, WanStats};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterApiError {
    #[error("The router API is unavailable or failed to respond.")]
    Unavailable,

    #[error("The router API returned an invalid response that could not be understood.")]
    InvalidResponse(String),

    #[error("Authentication with the router API failed. Please check credentials.")]
    AuthenticationFailed,

    #[error("An unknown error occurred while communicating with the router API.")]
    Unknown(String),
}

pub type RouterApiResult<T> = Result<T, RouterApiError>;

#[async_trait::async_trait]
pub trait RouterApi: Send + Sync {
    async fn wan_connectivity(&self) -> RouterApiResult<WanConnectivity>;
    async fn list_devices(&self) -> RouterApiResult<Vec<Device>>;
    async fn wan_stats(&self) -> RouterApiResult<WanStats>;
}
