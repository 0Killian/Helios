use uuid::Uuid;

use crate::{
    error::{ProtocolError, ProtocolResult},
    handshake::{expect_message_or_fail, random_nonce, resolve_challenge},
    message::{
        Message as ProtocolMessage, MessageContent, MessageResult,
        core::{self},
    },
};
use futures_util::{SinkExt, StreamExt};

/// Performs the agent-side handshake.
///
/// 1. Sends the `Authenticate` message with the agent's public `service_id`.
/// 2. Waits for the server's `Challenge` message.
/// 3. Parses the message and retrieves the `agent_nonce`.
/// 4. Resolves the challenge and sends the `ChallengeResponse` message.
/// 5. Waits for the server's `AuthenticateSuccess` message.
/// 6. Verifies the response and returns `Ok(())` on success.
///
/// # Arguments
///
/// * `ws_stream` - A mutable reference to the active WebSocket stream.
/// * `service_id` - The public UUID of the service this agent represents.
/// * `token` - The authentication token for the service.
pub async fn initiate_handshake<S>(
    ws_stream: &mut S,
    service_id: Uuid,
    token: &str,
) -> ProtocolResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    let auth_message = ProtocolMessage::new(MessageContent::Core(MessageResult::Ok(
        core::OkMessage::Authenticate { service_id },
    )));
    let id = auth_message.id;

    ws_stream.send(auth_message.try_into()?).await?;

    let nonce = match expect_message_or_fail(ws_stream, id).await?.content {
        MessageContent::Core(MessageResult::Ok(core::OkMessage::Challenge { agent_nonce })) => {
            agent_nonce
        }
        MessageContent::Core(MessageResult::Err(core::ErrorMessage::AgentNotFound)) => {
            return Err(ProtocolError::AgentNotFound);
        }
        _ => {
            return Err(ProtocolError::UnexpectedMessage);
        }
    };

    let agent_challenge_response = resolve_challenge(&nonce, &token);

    let server_nonce = random_nonce();
    let server_challenge_response = resolve_challenge(&server_nonce, &token);
    ws_stream
        .send(
            ProtocolMessage::respond(
                id,
                MessageContent::Core(MessageResult::Ok(core::OkMessage::ChallengeResponse {
                    response: agent_challenge_response.clone(),
                    server_nonce: server_nonce.clone(),
                })),
            )
            .try_into()?,
        )
        .await?;

    match expect_message_or_fail(ws_stream, id).await?.content {
        MessageContent::Core(MessageResult::Ok(core::OkMessage::AuthenticationSuccess {
            ref response,
        })) => {
            if *response != server_challenge_response {
                ws_stream
                    .send(
                        ProtocolMessage::respond(
                            id,
                            MessageContent::Core(MessageResult::Err(
                                core::ErrorMessage::AuthenticationFailed,
                            )),
                        )
                        .try_into()?,
                    )
                    .await?;
                return Err(ProtocolError::HandshakeFailed);
            }
        }
        MessageContent::Core(MessageResult::Err(core::ErrorMessage::AuthenticationFailed)) => {
            return Err(ProtocolError::HandshakeFailed);
        }
        _ => {
            return Err(ProtocolError::UnexpectedMessage);
        }
    };

    ws_stream
        .send(
            ProtocolMessage::respond(
                id,
                MessageContent::Core(MessageResult::Ok(core::OkMessage::HandshakeComplete)),
            )
            .try_into()?,
        )
        .await?;

    Ok(())
}
