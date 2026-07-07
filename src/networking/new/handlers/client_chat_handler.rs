use crate::networking::{
    chat::{ChatMessageType, ChatRequest},
    new::{
        client_message::ClientMessage, handlers::handler::ClientMessageHandler,
        message_header::MessageHeader, user_registry::UserRegistryHandle,
    },
};
use anyhow::anyhow;
use tokio::sync::broadcast;

pub struct ClientChatHandler {
    chat_tx: broadcast::Sender<ChatRequest>,
    users: UserRegistryHandle,
}

impl ClientChatHandler {
    pub fn new(chat_tx: broadcast::Sender<ChatRequest>, users: UserRegistryHandle) -> Self {
        Self { chat_tx, users }
    }
}

impl ClientMessageHandler for ClientChatHandler {
    fn accept_header(&self, header: &MessageHeader) -> bool {
        return header.bytes == *b"CHAT";
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        let message = String::from_utf8_lossy(&msg.data);

        if let Some(user_id) = self.users.client_to_user_id(msg.client_id) {
            let user_data = self.users.get(&user_id);

            let request = ChatRequest {
                message: message.to_string(),
                message_type: ChatMessageType::Normal,
                recipient_filter: None,
                sender_id: user_id,
                sender_name: user_data.map_or("[Anonymous]".to_owned(), |d| d.name),
            };

            let _ = self.chat_tx.send(request);

            Ok(())
        } else {
            Err(anyhow!("a"))
        }
    }
}
