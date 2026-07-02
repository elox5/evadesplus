use crate::networking::{
    chat::ChatRequest,
    helpers::create_server_announcement,
    leaderboard::LeaderboardUpdate,
    new::{
        client_message::ClientMessage, handlers::handler::ClientMessageHandler,
        message_header::MessageHeader, user_registry::UserRegistryHandle,
    },
};
use anyhow::anyhow;
use tokio::sync::broadcast;

pub struct CloseHandler {
    user_registry: UserRegistryHandle,
    lb_tx: broadcast::Sender<LeaderboardUpdate>,
    chat_tx: broadcast::Sender<ChatRequest>,
}

impl CloseHandler {
    pub fn new(
        user_registry: UserRegistryHandle,
        lb_tx: broadcast::Sender<LeaderboardUpdate>,
        chat_tx: broadcast::Sender<ChatRequest>,
    ) -> Self {
        Self {
            user_registry,
            lb_tx,
            chat_tx,
        }
    }
}

impl ClientMessageHandler for CloseHandler {
    fn accept_header(&self, header: &MessageHeader) -> bool {
        return header.bytes == *b"CLSE";
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        if let Some(user_id) = self.user_registry.client_to_user_id(msg.client_id) {
            let user = self.user_registry.get(&user_id).unwrap();
            self.user_registry.remove(&user_id);

            let chat_broadcast = create_server_announcement(format!("{} left the game", user.name));
            let _ = self.chat_tx.send(chat_broadcast);

            let lb_update = LeaderboardUpdate::remove(user_id);
            let _ = self.lb_tx.send(lb_update);

            return Ok(());
        }

        Err(anyhow!(""))
    }
}
