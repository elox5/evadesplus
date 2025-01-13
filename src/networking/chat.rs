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
        bytes.push(self.sender_name.len() as u8); // 1 byte
        bytes.push(self.message.len() as u8); // 1 byte
        bytes.extend_from_slice(self.sender_name.as_bytes()); // name.len() bytes
        bytes.extend_from_slice(self.message.as_bytes()); // message.len() bytes

        bytes
    }
}

#[derive(Clone)]
pub enum ChatMessageType {
    Normal,
    Whisper,
    CommandResponse,
    ServerAnnouncement,
    ServerError,
}
