use crate::networking::new::{
    client_message::{ClientMessage, MessageHeader},
    handlers::handler::ClientMessageHandler,
    user_registry::UserRegistryHandle,
};

pub struct InitHandler {
    user_registry: UserRegistryHandle,
}

impl InitHandler {
    pub fn new(user_registry: UserRegistryHandle) -> Self {
        Self { user_registry }
    }
}

impl ClientMessageHandler for InitHandler {
    fn accepted_headers(&self) -> Vec<MessageHeader> {
        return vec!["INIT".into()];
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        let name = String::from_utf8_lossy(&msg.data).to_string();

        self.user_registry.create_user(name, msg.client_id);

        Ok(())
    }
}
