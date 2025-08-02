use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Event {
    Ping,
}

#[derive(Debug, Error)]
pub enum AgentConnectionManagerError {
    #[error("Agent already registered")]
    AgentAlreadyRegistered,
    #[error("Agent not registered")]
    AgentNotRegistered,
    #[error("Send error")]
    SendError,
}
pub type AgentConnectionManagerResult<T> = Result<T, AgentConnectionManagerError>;

#[async_trait::async_trait]
pub trait AgentConnectionManager: Send + Sync {
    async fn dispatch_event(&self, agents: Uuid, event: Event) -> AgentConnectionManagerResult<()>;
    async fn broadcast_event(&self, event: Event) -> AgentConnectionManagerResult<()>;

    async fn register_agent(
        &self,
        agent: Uuid,
    ) -> AgentConnectionManagerResult<(mpsc::Receiver<Event>, broadcast::Receiver<Event>)>;
    async fn unregister_agent(&self, agent: Uuid) -> AgentConnectionManagerResult<()>;
}
