use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) mod core;

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
    Core(MessageResult<core::OkMessage, core::ErrorMessage>),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "status")]
pub enum MessageResult<OkMessage, ErrorMessage> {
    #[serde(rename = "ok")]
    Ok(OkMessage),
    #[serde(rename = "error")]
    Err(ErrorMessage),
}

impl Message {
    pub fn new(content: MessageContent) -> Self {
        Self {
            id: Uuid::now_v7(),
            content,
        }
    }

    pub fn respond(id: Uuid, content: MessageContent) -> Self {
        Self { id, content }
    }
}

impl TryInto<String> for Message {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self)
    }
}

impl FromStr for Message {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_core_serialization_deserialization() {
        let service_id = Uuid::now_v7();
        let message = Message {
            id: Uuid::now_v7(),
            content: MessageContent::Core(MessageResult::Ok(core::OkMessage::Authenticate {
                service_id,
            })),
        };

        let serialized = serde_json::to_string(&message).unwrap();

        assert_eq!(
            serialized,
            format!(
                "{{\"id\":\"{}\",\"namespace\":\"core\",\"status\":\"ok\",\"command\":\"Authenticate\",\"payload\":{{\"service_id\":\"{}\"}}}}",
                message.id, service_id
            )
        );

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(message, deserialized);
    }
}
