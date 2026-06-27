use crate::networking::new::client_message::{ClientMessage, MessageHeader};
use anyhow::Result;

pub trait ClientMessageHandler {
    fn accepted_headers(&self) -> Vec<MessageHeader>;
    fn handle(&self, msg: ClientMessage) -> Result<()>;
}
