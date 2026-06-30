use crate::{
    logger::{LogCategory, Logger},
    networking::new::{
        client_message::{ClientMessage, MessageHeader},
        handlers::handler::ClientMessageHandler,
    },
};

pub struct ClientMessageLogger {
    cat: LogCategory,
}

impl Default for ClientMessageLogger {
    fn default() -> Self {
        Self {
            cat: LogCategory::Network,
        }
    }
}

impl ClientMessageHandler for ClientMessageLogger {
    fn accept_header(&self, _header: &MessageHeader) -> bool {
        return true;
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        Logger::log(
            format!(
                "Received '{}' from client @{}",
                msg.header.to_string(),
                msg.client_id
            ),
            self.cat.clone(),
        );
        Ok(())
    }
}
