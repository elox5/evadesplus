use tokio::sync::broadcast;

use crate::networking::{
    chat::{ChatMessageType, ChatRequest},
    new::{
        client_message::{ClientMessage, MessageHeader},
        handlers::handler::ClientMessageHandler,
    },
};

pub struct ClientChatHandler {
    chat_tx: broadcast::Sender<ChatRequest>,
}

impl ClientChatHandler {
    pub fn new(chat_tx: broadcast::Sender<ChatRequest>) -> Self {
        Self { chat_tx }
    }
}

impl ClientMessageHandler for ClientChatHandler {
    fn accepted_headers(&self) -> Vec<MessageHeader> {
        return vec!["CHAT".into()];
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        let message = String::from_utf8_lossy(&msg.data);

        let request = ChatRequest {
            message: message.to_string(),
            message_type: ChatMessageType::Normal,
            recipient_filter: None,
            sender_id: msg.client_id as u64,
            sender_name: "Anonymous".to_owned(),
        };

        let _ = self.chat_tx.send(request);

        Ok(())
    }
}
