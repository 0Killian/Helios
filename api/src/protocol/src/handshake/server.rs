use anyhow::Result;

use crate::{
    error::{ProtocolError, ServerResult},
    handshake::{expect_message_or_fail, random_nonce, resolve_challenge},
    message::{
        Message as ProtocolMessage, MessageContent, MessageResult,
        core::{self},
    },
};
use futures_util::{SinkExt, StreamExt};

/// Performs the server-side handshake.
pub async fn accept_handshake<S>(
    ws_stream: &mut S,
    message: ProtocolMessage,
    token: &str,
) -> ServerResult<()>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    let id = message.id;
    let agent_nonce = random_nonce();
    let agent_challenge_response = resolve_challenge(&agent_nonce, &token);

    let challenge_response_message = ProtocolMessage::respond(
        id,
        MessageContent::Core(MessageResult::Ok(core::OkMessage::Challenge {
            agent_nonce: agent_nonce.clone(),
        })),
    );

    ws_stream
        .send(challenge_response_message.try_into()?)
        .await?;

    let server_nonce = match expect_message_or_fail(ws_stream, id).await?.content {
        MessageContent::Core(MessageResult::Ok(core::OkMessage::ChallengeResponse {
            ref response,
            server_nonce,
        })) => {
            if *response != agent_challenge_response {
                ws_stream
                    .send(
                        ProtocolMessage::respond(
                            id,
                            MessageContent::Core(MessageResult::Err(
                                core::ErrorMessage::AuthenticationFailed {},
                            )),
                        )
                        .try_into()?,
                    )
                    .await?;

                return Err(ProtocolError::HandshakeFailed.into());
            }
            server_nonce
        }
        _ => {
            ws_stream
                .send(
                    ProtocolMessage::respond(
                        id,
                        MessageContent::Core(MessageResult::Err(
                            core::ErrorMessage::InvalidMessage,
                        )),
                    )
                    .try_into()?,
                )
                .await?;

            return Err(ProtocolError::HandshakeFailed.into());
        }
    };

    let server_challenge_response = resolve_challenge(&server_nonce, &token);

    let server_challenge_response_message = ProtocolMessage::respond(
        id,
        MessageContent::Core(MessageResult::Ok(core::OkMessage::AuthenticationSuccess {
            response: server_challenge_response,
        })),
    );

    ws_stream
        .send(server_challenge_response_message.try_into()?)
        .await?;

    match expect_message_or_fail(ws_stream, id).await?.content {
        MessageContent::Core(MessageResult::Ok(core::OkMessage::HandshakeComplete)) => Ok(()),
        MessageContent::Core(MessageResult::Err(core::ErrorMessage::AuthenticationFailed)) => {
            Err(ProtocolError::HandshakeFailed.into())
        }
        _ => Err(ProtocolError::UnexpectedMessage.into()),
    }
}
