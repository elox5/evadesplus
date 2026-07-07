use tokio::sync::broadcast;

use crate::networking::new::user_registry::UserId;

pub struct Chat {
    pub tx: broadcast::Sender<ChatRequest>,
    pub rx: broadcast::Receiver<ChatRequest>,
}

impl Chat {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(8);

        Self { tx, rx }
    }
}

#[derive(Clone)]
pub struct ChatRequest {
    pub message: String,
    pub sender_name: String,
    pub sender_id: UserId,
    pub message_type: ChatMessageType,
    pub recipient_filter: Option<Vec<UserId>>,
}

impl ChatRequest {
    pub fn new(
        message: String,
        sender_name: String,
        sender_id: UserId,
        message_type: ChatMessageType,
        recipient_filter: Option<Vec<UserId>>,
    ) -> Self {
        Self {
            message,
            sender_name,
            sender_id,
            message_type,
            recipient_filter,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.message_type.clone() as u8); // 1 byte
        bytes.extend_from_slice(&self.sender_id.0.to_le_bytes()); // 8 bytes
        bytes.push(self.message.len() as u8); // 1 byte
        bytes.extend_from_slice(self.message.as_bytes()); // message.len() bytes

        if self.message_type == ChatMessageType::Whisper {
            if let Some(recipients) = &self.recipient_filter {
                let target = recipients.iter().find(|r| **r != self.sender_id);

                if let Some(target) = target {
                    bytes.extend_from_slice(&target.0.to_le_bytes()); // 8 bytes
                }
            }
        }

        bytes
    }
}

#[derive(Clone, PartialEq)]
pub enum ChatMessageType {
    Normal,
    Whisper,
    CommandResponse,
    ServerAnnouncement,
    ServerError,
}
