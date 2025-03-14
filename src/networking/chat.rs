use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::broadcast;

static CHAT: LazyLock<Arc<Mutex<Option<Chat>>>> = LazyLock::new(|| Arc::new(Mutex::new(None)));

pub struct Chat {
    pub tx: broadcast::Sender<ChatRequest>,
    pub rx: broadcast::Receiver<ChatRequest>,
}

impl Chat {
    pub fn init(tx: broadcast::Sender<ChatRequest>, rx: broadcast::Receiver<ChatRequest>) {
        let chat = Self { tx, rx };

        *CHAT.lock().unwrap() = Some(chat);
    }

    pub fn tx() -> broadcast::Sender<ChatRequest> {
        CHAT.lock().unwrap().as_ref().unwrap().tx.clone()
    }

    pub fn rx() -> broadcast::Receiver<ChatRequest> {
        CHAT.lock().unwrap().as_ref().unwrap().rx.resubscribe()
    }
}

#[derive(Clone)]
pub struct ChatRequest {
    pub message: String,
    pub sender_name: String,
    pub sender_id: u64,
    pub message_type: ChatMessageType,
    pub recipient_filter: Option<Vec<u64>>,
}

impl ChatRequest {
    pub fn new(
        message: String,
        sender_name: String,
        sender_id: u64,
        message_type: ChatMessageType,
        recipient_filter: Option<Vec<u64>>,
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

        bytes.extend_from_slice(b"CHBR"); // 4 bytes (chat broadcast)
        bytes.push(self.message_type.clone() as u8); // 1 byte
        bytes.extend_from_slice(&self.sender_id.to_le_bytes()); // 8 bytes
        bytes.push(self.message.len() as u8); // 1 byte
        bytes.extend_from_slice(self.message.as_bytes()); // message.len() bytes

        if self.message_type == ChatMessageType::Whisper {
            if let Some(recipients) = &self.recipient_filter {
                let target = recipients.iter().find(|r| **r != self.sender_id);

                if let Some(target) = target {
                    bytes.extend_from_slice(&target.to_le_bytes()); // 8 bytes
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
