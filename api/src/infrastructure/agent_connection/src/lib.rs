use std::{mem::forget, sync::Arc};

use dashmap::DashMap;
use ports::agent_connection::{
    AgentConnectionManager as ACM, AgentConnectionManagerError as AcmError,
    AgentConnectionManagerResult as AcmResult, Event,
};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

#[derive(Clone)]
pub struct InMemoryAgentConnectionManager {
    downstream_direct_channels: Arc<DashMap<Uuid, mpsc::Sender<Event>>>,
    downstream_event_bus_tx: broadcast::Sender<Event>,
}

impl InMemoryAgentConnectionManager {
    pub fn new() -> Self {
        let (downstream_event_bus_tx, downstream_event_bus_rx) = broadcast::channel(100);
        forget(downstream_event_bus_rx);
        Self {
            downstream_direct_channels: Arc::new(DashMap::new()),
            downstream_event_bus_tx,
        }
    }
}

#[async_trait::async_trait]
impl ACM for InMemoryAgentConnectionManager {
    async fn dispatch_event(&self, agent: Uuid, event: Event) -> AcmResult<()> {
        if let Some(tx) = self.downstream_direct_channels.get(&agent) {
            tx.send(event).await.map_err(|_| AcmError::SendError)
        } else {
            Err(AcmError::AgentNotRegistered)
        }
    }

    async fn broadcast_event(&self, event: Event) -> AcmResult<()> {
        self.downstream_event_bus_tx
            .send(event)
            .map_err(|_| AcmError::SendError)
            .map(|_| ())
    }

    async fn register_agent(
        &self,
        agent: Uuid,
    ) -> AcmResult<(mpsc::Receiver<Event>, broadcast::Receiver<Event>)> {
        if self.downstream_direct_channels.contains_key(&agent) {
            return Err(AcmError::AgentAlreadyRegistered);
        }

        let (tx, rx) = mpsc::channel(100);
        self.downstream_direct_channels.insert(agent, tx);

        Ok((rx, self.downstream_event_bus_tx.subscribe()))
    }

    async fn unregister_agent(&self, agent: Uuid) -> AcmResult<()> {
        self.downstream_direct_channels.remove(&agent);
        // TODO: Send an event upstream
        Ok(())
    }
}
