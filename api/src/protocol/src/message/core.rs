use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    error::{AgentError, AgentResult, ServerResult},
    message::{Message, MessageContent, MessageResult},
};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "command", content = "payload")]
pub enum OkMessage {
    Authenticate {
        service_id: Uuid,
    },
    Challenge {
        agent_nonce: Nonce,
    },
    ChallengeResponse {
        response: String,
        server_nonce: Nonce,
    },
    AuthenticationSuccess {
        response: String,
    },
    HandshakeComplete,
    Ping,
    Pong,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "command", content = "payload")]
pub enum ErrorMessage {
    AgentNotFound,
    AuthenticationFailed,
    UnexpectedOutOfBandMessage,
    InternalError,
    InvalidMessage,
    AlreadyConnected,
}

pub type Nonce = [u8; 32];

pub async fn handle_server_message<S>(
    ws: &mut S,
    id: Uuid,
    message: MessageResult<OkMessage, ErrorMessage>,
) -> ServerResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    match message {
        MessageResult::Ok(
            OkMessage::Authenticate { .. }
            | OkMessage::Challenge { .. }
            | OkMessage::AuthenticationSuccess { .. }
            | OkMessage::HandshakeComplete
            | OkMessage::ChallengeResponse { .. },
        ) => {
            ws.send(
                Message::respond(
                    id,
                    MessageContent::Core(MessageResult::Err(ErrorMessage::InvalidMessage)),
                )
                .try_into()?,
            )
            .await?
        }
        MessageResult::Ok(OkMessage::Ping) => {
            ws.send(
                Message::respond(id, MessageContent::Core(MessageResult::Ok(OkMessage::Pong)))
                    .try_into()?,
            )
            .await?
        }
        MessageResult::Ok(OkMessage::Pong) => {}
        MessageResult::Err(ErrorMessage::UnexpectedOutOfBandMessage) => {
            println!("Unexpected Out-of-band message!");
        }
        MessageResult::Err(ErrorMessage::InternalError) => {
            println!("Internal error occurred!");
        }
        MessageResult::Err(ErrorMessage::InvalidMessage) => {
            println!("Invalid message sent!");
        }
        MessageResult::Err(
            ErrorMessage::AgentNotFound
            | ErrorMessage::AuthenticationFailed
            | ErrorMessage::AlreadyConnected,
        ) => {
            ws.send(
                Message::respond(
                    id,
                    MessageContent::Core(MessageResult::Err(ErrorMessage::InvalidMessage)),
                )
                .try_into()?,
            )
            .await?
        }
    }
    Ok(())
}

pub async fn handle_client_message<S>(
    ws: &mut S,
    id: Uuid,
    message: MessageResult<OkMessage, ErrorMessage>,
) -> AgentResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    match message {
        MessageResult::Ok(
            OkMessage::Authenticate { .. }
            | OkMessage::Challenge { .. }
            | OkMessage::AuthenticationSuccess { .. }
            | OkMessage::HandshakeComplete
            | OkMessage::ChallengeResponse { .. },
        ) => {
            ws.send(
                Message::respond(
                    id,
                    MessageContent::Core(MessageResult::Err(ErrorMessage::InvalidMessage)),
                )
                .try_into()?,
            )
            .await?
        }
        MessageResult::Ok(OkMessage::Ping) => {
            info!("Pong!");
            ws.send(
                Message::respond(id, MessageContent::Core(MessageResult::Ok(OkMessage::Pong)))
                    .try_into()?,
            )
            .await?
        }
        MessageResult::Ok(OkMessage::Pong) => {}
        MessageResult::Err(ErrorMessage::UnexpectedOutOfBandMessage) => {
            println!("Unexpected Out-of-band message!");
        }
        MessageResult::Err(ErrorMessage::InternalError) => {
            println!("Internal error occurred!");
        }
        MessageResult::Err(ErrorMessage::InvalidMessage) => {
            println!("Invalid message sent!");
        }
        MessageResult::Err(ErrorMessage::AgentNotFound | ErrorMessage::AuthenticationFailed) => {
            ws.send(
                Message::respond(
                    id,
                    MessageContent::Core(MessageResult::Err(ErrorMessage::InvalidMessage)),
                )
                .try_into()?,
            )
            .await?
        }
        MessageResult::Err(ErrorMessage::AlreadyConnected) => {
            return Err(AgentError::AlreadyConnected);
        }
    }
    Ok(())
}
