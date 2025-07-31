use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Message {
    pub id: Uuid,
    #[serde(flatten)]
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "namespace")]
pub enum MessageContent {
    #[serde(rename = "core")]
    Core(CoreMessageResult),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "status")]
pub enum CoreMessageResult {
    #[serde(rename = "ok")]
    Ok(CoreMessage),
    #[serde(rename = "error")]
    Error(CoreError),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "command", content = "payload")]
pub enum CoreMessage {
    Authenticate {},
    Challenge {},
    ChallengeResponse {},
    AuthenticationSuccess {},
    Ping {},
    Pong {},
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "command", content = "payload")]
pub enum CoreError {
    AgentNotFound {},
    ChallengeFailed {},
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_core_serialization_deserialization() {
        let message = Message {
            id: Uuid::now_v7(),
            content: MessageContent::Core(CoreMessageResult::Ok(CoreMessage::Authenticate {})),
        };

        let serialized = serde_json::to_string(&message).unwrap();

        assert_eq!(
            serialized,
            format!(
                "{{\"id\":\"{}\",\"namespace\":\"core\",\"status\":\"ok\",\"command\":\"Authenticate\",\"payload\":{{}}}}",
                message.id
            )
        );

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }
}
