use crate::networking::new::{client_id::ClientId, message_header::MessageHeader};

#[derive(Clone)]
pub struct ClientMessage {
    pub header: MessageHeader,
    pub client_id: ClientId,
    pub data: Vec<u8>,
}

impl ClientMessage {
    pub fn new(
        client_id: ClientId,
        header: impl Into<MessageHeader>,
        data: Vec<u8>,
    ) -> ClientMessage {
        ClientMessage {
            header: header.into(),
            client_id,
            data,
        }
    }
}
