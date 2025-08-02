use ports::{agent_connection::AgentConnectionManagerError, repositories::RepositoryError};
use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Agent connection manager error: {0}")]
    AgentConnectionManagerError(AgentConnectionManagerError),
    #[error("Stream error: {0}")]
    StreamError(#[from] std::io::Error),
    #[error("Message parse error: {0}")]
    MessageParseError(#[from] serde_json::Error),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Unexpected message")]
    UnexpectedMessage,
    #[error("Handshake failed")]
    HandshakeFailed,
    #[error("Agent not found")]
    AgentNotFound,
    #[error("Timeout")]
    Timeout,
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("Event broker disconnected")]
    EventBrokerDisconnected,
    #[error("Event broker receive error")]
    EventBrokerBroadcastRecvError(#[from] RecvError),
    #[error("Agent did not respond to a ping request in time")]
    PingFailed,
}

pub type ServerResult<T> = Result<T, ServerError>;

impl From<RepositoryError> for ServerError {
    fn from(error: RepositoryError) -> Self {
        ServerError::ProtocolError(ProtocolError::RepositoryError(error))
    }
}

impl From<AgentConnectionManagerError> for ServerError {
    fn from(error: AgentConnectionManagerError) -> Self {
        ServerError::ProtocolError(ProtocolError::AgentConnectionManagerError(error))
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(error: serde_json::Error) -> Self {
        ServerError::ProtocolError(ProtocolError::MessageParseError(error))
    }
}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        ServerError::ProtocolError(ProtocolError::StreamError(error))
    }
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("This agent is already connected")]
    AlreadyConnected,
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
}

pub type AgentResult<T> = Result<T, AgentError>;

impl From<RepositoryError> for AgentError {
    fn from(error: RepositoryError) -> Self {
        AgentError::ProtocolError(ProtocolError::RepositoryError(error))
    }
}

impl From<AgentConnectionManagerError> for AgentError {
    fn from(error: AgentConnectionManagerError) -> Self {
        AgentError::ProtocolError(ProtocolError::AgentConnectionManagerError(error))
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(error: serde_json::Error) -> Self {
        AgentError::ProtocolError(ProtocolError::MessageParseError(error))
    }
}

impl From<std::io::Error> for AgentError {
    fn from(error: std::io::Error) -> Self {
        AgentError::ProtocolError(ProtocolError::StreamError(error))
    }
}
