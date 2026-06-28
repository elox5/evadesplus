use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::networking::{
    leaderboard::LeaderboardStore,
    new::{
        client_message::{ClientMessage, MessageHeader},
        handlers::handler::ClientMessageHandler,
        server_message::{ServerMessage, ServerMessageTarget},
        user_registry::UserRegistryHandle,
    },
};

pub struct InitHandler {
    user_registry: UserRegistryHandle,
    server_tx: mpsc::Sender<ServerMessage>,
    lb_store: Arc<Mutex<LeaderboardStore>>,
}

impl InitHandler {
    pub fn new(
        user_registry: UserRegistryHandle,
        server_tx: mpsc::Sender<ServerMessage>,
        lb_store: Arc<Mutex<LeaderboardStore>>,
    ) -> Self {
        Self {
            user_registry,
            server_tx,
            lb_store,
        }
    }
}

impl ClientMessageHandler for InitHandler {
    fn accepted_headers(&self) -> Vec<MessageHeader> {
        return vec!["INIT".into()];
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        let name = String::from_utf8_lossy(&msg.data).to_string();

        let user_id = self.user_registry.create_user(name, msg.client_id);
        let store = self.lb_store.try_lock().unwrap(); // FIX lol

        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(0);
        bytes.extend_from_slice(&user_id.0.to_le_bytes());
        bytes.extend_from_slice(&store.to_bytes());

        let response = ServerMessage {
            header: "INIT".into(),
            data: bytes,
            target: ServerMessageTarget::Single(msg.client_id),
        };

        let _ = self.server_tx.try_send(response);

        Ok(())
    }
}
