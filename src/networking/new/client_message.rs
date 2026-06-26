use crate::networking::new::client_id::ClientId;

#[derive(Clone)]
pub struct ClientMessage {
    pub header: String,
    pub client_id: ClientId,
    pub data: Vec<u8>,
}

impl ClientMessage {
    pub fn new(client_id: ClientId, header: &str, data: Vec<u8>) -> ClientMessage {
        ClientMessage {
            header: header.to_owned(),
            client_id,
            data,
        }
    }
}
