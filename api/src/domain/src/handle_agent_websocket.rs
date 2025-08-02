use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use ports::{
    agent_connection::AgentConnectionManager,
    repositories::{ServicesRepository, UnitOfWorkProvider},
};
use tracing::{error, info};

#[derive(Clone)]
pub struct HandleAgentWebsocketUseCase<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> {
    uow_provider: UWP,
    acm: Arc<dyn AgentConnectionManager>,
    _marker: std::marker::PhantomData<SR>,
}

impl<SR: ServicesRepository<UWP>, UWP: UnitOfWorkProvider> HandleAgentWebsocketUseCase<SR, UWP> {
    pub fn new(
        uow_provider: UWP,
        agent_connection_manager: Arc<dyn AgentConnectionManager>,
    ) -> Self {
        Self {
            uow_provider,
            acm: agent_connection_manager,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn execute<S>(self, mut ws: S)
    where
        S: SinkExt<String, Error = std::io::Error>
            + StreamExt<Item = Result<String, std::io::Error>>
            + Unpin,
    {
        info!("Connecting websocket");
        match protocol::handle_server::<_, SR, _>(&mut ws, &self.uow_provider, self.acm.clone())
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!("Error handling agent websocket: {}", err);
            }
        }

        match ws.flush().await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to flush websocket properly: {}", err);
            }
        }

        match ws.close().await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to close websocket properly: {}", err);
            }
        }

        match ws.flush().await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to flush websocket properly: {}", err);
            }
        }
    }
}
