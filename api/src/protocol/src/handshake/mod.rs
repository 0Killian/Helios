use crate::message::core;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

mod client;
mod server;

pub fn resolve_challenge(nonce: &[u8; 32], token: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(token.as_bytes()).expect("HMAC can take key of any size");
    mac.update(nonce);
    hex::encode(mac.finalize().into_bytes())
}

pub fn random_nonce() -> Nonce {
    let mut rng = rand::rng();
    let mut nonce = [0u8; 32];
    rng.fill(&mut nonce);
    nonce
}

pub async fn expect_message_or_fail<S>(ws: &mut S, id: Uuid) -> ProtocolResult<Message>
where
    S: SinkExt<String, Error = std::io::Error>
        + StreamExt<Item = Result<String, std::io::Error>>
        + Unpin,
{
    expect_message(
        ws,
        id,
        async |ws, msg| {
            ws.send(
                Message::respond(
                    msg.id,
                    MessageContent::Core(MessageResult::Err(
                        core::ErrorMessage::UnexpectedOutOfBandMessage,
                    )),
                )
                .try_into()?,
            )
            .await?;
            Err(ProtocolError::UnexpectedMessage)
        },
        std::time::Duration::MAX,
    )
    .await
}

pub(crate) use client::initiate_handshake;
pub(crate) use server::accept_handshake;
use uuid::Uuid;

use crate::{
    error::{ProtocolError, ProtocolResult},
    expect_message,
    message::{Message, MessageContent, MessageResult, core::Nonce},
};

// #[cfg(test)]
// mod tests {
//     use futures_util::{SinkExt, StreamExt};
//     use tokio::io::duplex;
//     use uuid::Uuid;

//     use crate::{
//         message::{Message, MessageContent, MessageResult, core},
//         receive_message,
//     };

//     use super::*;

//     #[tokio::test]
//     async fn test_handshake_success() {
//         // Create an in-memory channel that acts like a network connection.
//         let (client_ws, server_ws) = duplex(1024);

//         let service_id = Uuid::now_v7();
//         let token = "my-test-token";

//         // Spawn the server task.
//         // It needs to receive the first message before starting its handshake logic.
//         let server_handle = tokio::spawn(async move {
//             let first_msg = server_ws.next().await.unwrap().unwrap();
//             let protocol_msg = Message::from(first_msg);

//             // Ensure the first message is an Authenticate message
//             match protocol_msg.content {
//                 MessageContent::Core(MessageResult::Ok(core::OkMessage::Authenticate {
//                     ..
//                 })) => {
//                     // Proceed with the handshake
//                     accept_handshake(&mut server_ws, protocol_msg, token).await
//                 }
//                 _ => Err(anyhow::anyhow!(
//                     "Server did not receive Authenticate message first."
//                 )),
//             }
//         });

//         // Run the agent handshake on the main test thread.
//         let agent_result = initiate_handshake(&mut agent_ws, service_id, token).await;

//         // Await the server task and check its result.
//         let server_result = server_handle.await.unwrap();

//         // Assert that both sides completed successfully.
//         assert!(
//             agent_result.is_ok(),
//             "Agent handshake failed: {:?}",
//             agent_result.err()
//         );
//         assert!(
//             server_result.is_ok(),
//             "Server handshake failed: {:?}",
//             server_result.err()
//         );
//     }

//     #[tokio::test]
//     async fn test_handshake_server_rejects_invalid_agent_token() {
//         let (client_stream, server_stream) = duplex(1024);
//         let mut agent_ws =
//             WebSocketStream::from_raw_socket(client_stream, Role::Client, None).await;
//         let mut server_ws =
//             WebSocketStream::from_raw_socket(server_stream, Role::Server, None).await;

//         let service_id = Uuid::now_v7();
//         let correct_token = "my-correct-token";
//         let wrong_token = "my-wrong-token";

//         let server_handle = tokio::spawn(async move {
//             let first_msg = server_ws.next().await.unwrap().unwrap();
//             let protocol_msg = Message::from(first_msg);
//             accept_handshake(&mut server_ws, protocol_msg, correct_token).await
//         });

//         let agent_result = initiate_handshake(&mut agent_ws, service_id, wrong_token).await;

//         let server_result = server_handle.await.unwrap();

//         assert!(
//             agent_result.is_err(),
//             "Agent should have failed, server should have detected the error"
//         );

//         assert!(server_result.is_err());
//         assert!(
//             server_result
//                 .unwrap_err()
//                 .to_string()
//                 .contains("response mismatch")
//         );
//     }

//     #[tokio::test]
//     async fn test_handshake_agent_rejects_impostor_server() {
//         let (client_stream, server_stream) = duplex(1024);
//         let mut agent_ws =
//             WebSocketStream::from_raw_socket(client_stream, Role::Client, None).await;
//         let mut server_ws =
//             WebSocketStream::from_raw_socket(server_stream, Role::Server, None).await;
//         let service_id = Uuid::now_v7();
//         let correct_token = "the-real-secret";
//         let impostor_token = "a-fake-secret";

//         // Spawn a malicious server task that will send a bad signature
//         let server_handle = tokio::spawn(async move {
//             let auth_msg = server_ws.next().await.unwrap().unwrap();
//             let auth_protocol_msg = Message::from(auth_msg);

//             let agent_nonce = random_nonce();
//             let challenge_msg = Message::respond(
//                 &auth_protocol_msg,
//                 MessageContent::Core(MessageResult::Ok(core::OkMessage::Challenge {
//                     agent_nonce,
//                 })),
//             );
//             server_ws.send(challenge_msg.into()).await.unwrap();

//             let challenge_resp_msg = receive_message(&mut server_ws).await.unwrap();

//             // Ignore the agent's signature and get the server_nonce.
//             let server_nonce = match challenge_resp_msg.content {
//                 MessageContent::Core(MessageResult::Ok(core::OkMessage::ChallengeResponse {
//                     server_nonce,
//                     ..
//                 })) => server_nonce,
//                 _ => panic!("Impostor server received unexpected message"),
//             };

//             let bad_server_signature = resolve_challenge(&server_nonce, &impostor_token);
//             let auth_success_msg = Message::respond(
//                 &challenge_resp_msg,
//                 MessageContent::Core(MessageResult::Ok(core::OkMessage::AuthenticationSuccess {
//                     response: bad_server_signature,
//                 })),
//             );
//             server_ws.send(auth_success_msg.into()).await.unwrap();
//         });

//         let agent_result = initiate_handshake(&mut agent_ws, service_id, correct_token).await;

//         assert!(agent_result.is_err());
//         assert!(
//             agent_result
//                 .unwrap_err()
//                 .to_string()
//                 .contains("Server challenge response mismatch")
//         );

//         server_handle.await.unwrap();
//     }
// }
