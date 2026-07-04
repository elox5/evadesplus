use crate::networking::{
    new::{
        server_message::{ServerMessage, ServerMessageTarget},
        user_registry::{UserData, UserRegistryHandle},
    },
    rendering::{AreaDefinitionMessage, AreaRenderMessage, AreaRenderPacket},
};
use tokio::sync::mpsc;

pub struct RenderHandler {
    pub users: UserRegistryHandle,
    pub server_tx: mpsc::Sender<ServerMessage>,
}

impl RenderHandler {
    pub async fn handle_render(&self, message: AreaRenderMessage) {
        let users = self.users.get_all();
        let targets: Vec<&UserData> = users
            .iter()
            .filter(|u| u.player_id.area == message.key)
            .collect();

        let message = Self::build_render_message(
            targets,
            message.enrich(self.users.player_to_user_id_map()).packet,
        );

        let _ = self.server_tx.send(message).await;
    }

    pub async fn handle_area_definition(&self, message: AreaDefinitionMessage) {
        if let Some(user_id) = self.users.player_to_user_id(message.id) {
            if let Some(user) = self.users.get(&user_id) {
                //

                let message = ServerMessage {
                    header: "ADEF".into(),
                    data: message.data,
                    target: ServerMessageTarget::Single(user.client_id.unwrap()),
                };

                let _ = self.server_tx.send(message).await;
            }
        }
    }

    fn build_render_message(targets: Vec<&UserData>, packet: AreaRenderPacket) -> ServerMessage {
        ServerMessage {
            header: "REND".into(),
            data: packet.to_bytes(),
            target: ServerMessageTarget::Group(targets.iter().flat_map(|d| d.client_id).collect()),
        }
    }
}
