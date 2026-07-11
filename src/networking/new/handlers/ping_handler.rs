use tokio::sync::mpsc::Sender;

use crate::networking::new::{
    client_message::ClientMessage,
    message_header::MessageHeader,
    server_message::{ServerMessage, ServerMessageTarget},
};

pub struct PingHandler {
    server_tx: Sender<ServerMessage>,
}

impl PingHandler {
    pub fn new(server_tx: Sender<ServerMessage>) -> Self {
        Self { server_tx }
    }
}

impl PingHandler {
    pub fn accept_header(&self, header: &MessageHeader) -> bool {
        return header.bytes == *b"PING";
    }

    pub async fn handle(&self, msg: ClientMessage) -> anyhow::Result<()> {
        self.server_tx
            .send(ServerMessage {
                header: "PONG".into(),
                data: Vec::new(),
                target: ServerMessageTarget::Single(msg.client_id),
            })
            .await?;

        Ok(())
    }
}
