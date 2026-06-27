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
    fn accepted_headers() -> Vec<MessageHeader> {
        return vec!["CHAT".into(), "INIT".into()];
    }

    fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        Logger::log(msg.header.to_string(), self.cat.clone());
        Ok(())
    }
}
