use std::{str::FromStr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use ports::{
    agent_connection::{AgentConnectionManager, AgentConnectionManagerError, Event},
    repositories::{RepositoryError, ServicesRepository, UnitOfWorkProvider},
};
use tokio::{
    select,
    sync::{broadcast, mpsc},
};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    error::{AgentResult, ProtocolError, ProtocolResult, ServerError, ServerResult},
    handshake::{accept_handshake, initiate_handshake},
    message::{Message, MessageContent, MessageResult, core},
};

pub mod error;
mod handshake;
mod message;

#[inline]
async fn receive_message<S>(ws_stream: &mut S) -> ProtocolResult<Message>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    let Some(string) = ws_stream.next().await else {
        return Err(ProtocolError::StreamError(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "WebSocket stream closed unexpectedly",
        )));
    };

    let message = Message::from_str(&string?)?;
    Ok(message)
}

pub async fn handle_server<S, SR, UWP>(
    ws: &mut S,
    uow_provider: &UWP,
    acm: Arc<dyn AgentConnectionManager>,
) -> ServerResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
    SR: ServicesRepository<UWP>,
    UWP: UnitOfWorkProvider,
{
    let message = receive_message(ws).await?;
    let mut uow = uow_provider.begin_transaction().await?;

    let service_id = match message.content {
        MessageContent::Core(MessageResult::Ok(core::OkMessage::Authenticate { service_id })) => {
            service_id
        }
        _ => {
            ws.send(
                Message::respond(
                    message.id,
                    MessageContent::Core(MessageResult::Err(core::ErrorMessage::InvalidMessage)),
                )
                .try_into()?,
            )
            .await?;
            return Err(ProtocolError::UnexpectedMessage.into());
        }
    };

    let service = match SR::fetch_one(&mut uow, service_id).await {
        Ok(service) => service,
        Err(RepositoryError::NotFound) => {
            ws.send(
                Message::respond(
                    message.id,
                    MessageContent::Core(MessageResult::Err(core::ErrorMessage::AgentNotFound)),
                )
                .try_into()?,
            )
            .await?;
            return Err(ProtocolError::AgentNotFound.into());
        }
        Err(error) => {
            ws.send(
                Message::respond(
                    message.id,
                    MessageContent::Core(MessageResult::Err(core::ErrorMessage::InternalError)),
                )
                .try_into()?,
            )
            .await?;
            return Err(error.into());
        }
    };

    accept_handshake(ws, message, &service.token).await?;

    let (agent_rx, broadcast_rx) = match acm.register_agent(service_id).await {
        Ok(res) => res,
        Err(AgentConnectionManagerError::AgentAlreadyRegistered) => {
            ws.send(
                Message::new(MessageContent::Core(MessageResult::Err(
                    core::ErrorMessage::AlreadyConnected,
                )))
                .try_into()?,
            )
            .await?;
            return Err(AgentConnectionManagerError::AgentAlreadyRegistered.into());
        }
        Err(e) => {
            ws.send(
                Message::new(MessageContent::Core(MessageResult::Err(
                    core::ErrorMessage::InternalError,
                )))
                .try_into()?,
            )
            .await?;
            return Err(e.into());
        }
    };

    pub async fn server_loop<S>(
        ws: &mut S,
        mut agent_rx: mpsc::Receiver<Event>,
        mut broadcast_rx: broadcast::Receiver<Event>,
    ) -> ServerResult<()>
    where
        S: SinkExt<String, Error = std::io::Error>
            + StreamExt<Item = Result<String, std::io::Error>>
            + Unpin,
    {
        loop {
            let ws_message = receive_message(ws);
            let internal_rx_event = agent_rx.recv();
            let internal_broadcast_rx_event = broadcast_rx.recv();
            select! {
                event = internal_rx_event => {
                    handle_server_event(ws, event.ok_or(ServerError::EventBrokerDisconnected)?).await?;
                }
                message = internal_broadcast_rx_event => {
                    handle_server_event(ws, message?).await?;
                }
                message = ws_message => {
                    debug!(?message, "Received message");
                    handle_server_message(ws, message?).await?;
                }
            }
        }
    }

    let result = server_loop(ws, agent_rx, broadcast_rx).await;

    acm.unregister_agent(service_id).await?;
    result
}

pub async fn handle_agent<S>(ws: &mut S, service_id: Uuid, token: &str) -> AgentResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    initiate_handshake(ws, service_id, token).await?;

    info!("Receiving messages");
    loop {
        let message = receive_message(ws).await?;
        debug!(?message, "Received message");
        handle_client_message(ws, message).await?;
    }
}

pub async fn handle_server_message<S>(ws: &mut S, message: Message) -> ServerResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    match message.content {
        MessageContent::Core(c) => core::handle_server_message(ws, message.id, c).await,
    }
}

pub async fn handle_client_message<S>(ws: &mut S, message: Message) -> AgentResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    match message.content {
        MessageContent::Core(c) => core::handle_client_message(ws, message.id, c).await,
    }
}

pub async fn expect_message<S, E>(
    ws: &mut S,
    id: Uuid,
    mut default_handler: impl AsyncFnMut(&mut S, Message) -> Result<(), E>,
    mut timeout: std::time::Duration,
) -> Result<Message, E>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
    E: From<ProtocolError>,
{
    loop {
        let now = std::time::Instant::now();
        select! {
            message = receive_message(ws) => {
                let message = message?;

                if message.id == id {
                    return Ok(message);
                } else {
                    default_handler(ws, message).await?;
                }

                timeout -= std::time::Instant::now() - now;
            }
            _ = tokio::time::sleep(timeout) => {
                return Err(ProtocolError::Timeout.into())
            }
        }
    }
}

pub async fn handle_server_event<S>(ws: &mut S, event: Event) -> ServerResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    match event {
        Event::Ping => {
            info!("Sending ping!");
            let message = Message::new(MessageContent::Core(MessageResult::Ok(
                core::OkMessage::Ping,
            )));
            let id = message.id;
            ws.send(message.try_into()?).await?;

            if !matches!(
                expect_message(
                    ws,
                    id,
                    handle_server_message,
                    std::time::Duration::from_secs(5)
                )
                .await?,
                Message {
                    content: MessageContent::Core(MessageResult::Ok(core::OkMessage::Pong)),
                    ..
                }
            ) {
                ws.send(
                    Message::respond(
                        id,
                        MessageContent::Core(MessageResult::Err(
                            core::ErrorMessage::InvalidMessage,
                        )),
                    )
                    .try_into()?,
                )
                .await?;
                return Err(ProtocolError::UnexpectedMessage.into());
            }

            info!("Pong received!");
        }
    }

    Ok(())
}
