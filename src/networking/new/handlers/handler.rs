use crate::networking::new::{client_message::ClientMessage, message_header::MessageHeader};
use anyhow::Result;

pub trait ClientMessageHandler {
    fn accept_header(&self, header: &MessageHeader) -> bool;
    fn handle(&self, msg: ClientMessage) -> Result<()>;
}
