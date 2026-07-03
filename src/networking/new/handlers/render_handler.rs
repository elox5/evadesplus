use crate::networking::{
    new::{
        server_message::{ServerMessage, ServerMessageTarget},
        user_registry::{UserData, UserRegistryHandle},
    },
    rendering::{AreaRenderMessage, AreaRenderPacket},
};
use tokio::sync::mpsc;

pub struct RenderHandler {
    pub users: UserRegistryHandle,
    pub server_tx: mpsc::Sender<ServerMessage>,
}

impl RenderHandler {
    pub async fn handle(&self, message: AreaRenderMessage) {
        let users = self.users.get_all();
        let targets: Vec<&UserData> = users
            .iter()
            .filter(|u| u.player_id.area == message.key)
            .collect();

        let message = Self::build_message(targets, message.packet);

        let _ = self.server_tx.send(message).await;
    }

    fn build_message(targets: Vec<&UserData>, packet: AreaRenderPacket) -> ServerMessage {
        ServerMessage {
            header: "REND".into(),
            data: packet.to_bytes(),
            target: ServerMessageTarget::Group(targets.iter().flat_map(|d| d.client_id).collect()),
        }
    }
}
