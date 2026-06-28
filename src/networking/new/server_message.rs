use crate::networking::new::{client_id::ClientId, client_message::MessageHeader};

#[derive(Clone, Debug)]
pub enum ServerMessageTarget {
    Single(ClientId),
    Group(Vec<ClientId>),
    All,
}

#[derive(Clone)]
pub struct ServerMessage {
    pub header: MessageHeader,
    pub data: Vec<u8>,
    pub target: ServerMessageTarget,
}
