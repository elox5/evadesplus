use crate::{
    game::game::GameHandle,
    networking::new::{
        client_message::ClientMessage, message_header::MessageHeader,
        user_registry::UserRegistryHandle,
    },
    physics::vec2::Vec2,
};

pub struct MoveHandler {
    users: UserRegistryHandle,
    game: GameHandle,
}

impl MoveHandler {
    pub fn new(users: UserRegistryHandle, game: GameHandle) -> Self {
        Self { users, game }
    }
}

impl MoveHandler {
    pub fn accept_header(&self, header: &MessageHeader) -> bool {
        return header.bytes == *b"MOVE";
    }

    pub async fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        let data = msg.data;
        let x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let y = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        if let Some(user_id) = self.users.client_to_user_id(msg.client_id) {
            if let Some(user) = self.users.get(&user_id) {
                let _ = self
                    .game
                    .send_input_update(user.player_id, Vec2::new(x, y))
                    .await;
            }
        }

        Ok(())
    }
}
