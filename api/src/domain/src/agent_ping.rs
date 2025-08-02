use std::sync::Arc;

use ports::agent_connection::{AgentConnectionManager, Event};
use tracing::error;

use crate::PeriodicUseCase;

#[derive(Clone)]
pub struct AgentPing {
    acm: Arc<dyn AgentConnectionManager>,
}

impl AgentPing {
    pub fn new(acm: Arc<dyn AgentConnectionManager>) -> Self {
        Self { acm }
    }
}

#[async_trait::async_trait]
impl PeriodicUseCase for AgentPing {
    fn next_execution(&self) -> Option<std::time::Instant> {
        Some(std::time::Instant::now() + std::time::Duration::from_secs(15))
    }

    async fn execute(&self) {
        match self.acm.broadcast_event(Event::Ping).await {
            Ok(()) => (),
            Err(e) => error!("Failed to broadcast ping event: {e}!"),
        };
    }
}
