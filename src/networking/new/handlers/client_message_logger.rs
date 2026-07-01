use crate::{
    logger::{LogCategory, Logger},
    networking::new::{
        client_message::ClientMessage, handlers::handler::ClientMessageHandler,
        message_header::MessageHeader,
    },
};

pub struct ClientMessageLogger {
    filter: Vec<String>,
    cat: LogCategory,
}

impl ClientMessageLogger {
    pub fn new(filter: Vec<String>) -> Self {
        Self {
            filter,
            cat: LogCategory::Network,
        }
    }
}

impl Default for ClientMessageLogger {
    fn default() -> Self {
        Self {
            cat: LogCategory::Network,
            filter: Vec::new(),
        }
    }
}

impl ClientMessageHandler for ClientMessageLogger {
    fn accept_header(&self, header: &MessageHeader) -> bool {
        return !self.filter.contains(&header.to_string());
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        Logger::log(
            format!(
                "Received '{}' from client {}",
                msg.header.to_string(),
                msg.client_id
            ),
            self.cat.clone(),
        );
        Ok(())
    }
}
