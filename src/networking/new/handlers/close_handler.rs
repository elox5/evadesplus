use crate::networking::{
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
}

impl CloseHandler {
    pub fn new(
        user_registry: UserRegistryHandle,
        lb_tx: broadcast::Sender<LeaderboardUpdate>,
    ) -> Self {
        Self {
            user_registry,
            lb_tx,
        }
    }
}

impl ClientMessageHandler for CloseHandler {
    fn accept_header(&self, header: &MessageHeader) -> bool {
        return header.bytes == *b"CLSE";
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        if let Some(user_id) = self.user_registry.client_to_user_id(msg.client_id) {
            self.user_registry.remove(&user_id);

            let lb_update = LeaderboardUpdate::remove(user_id.0);
            let _ = self.lb_tx.send(lb_update);

            return Ok(());
        }

        Err(anyhow!(""))
    }
}
