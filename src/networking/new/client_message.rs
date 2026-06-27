use crate::networking::new::client_id::ClientId;

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

#[derive(Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub header: [u8; 4],
}

impl MessageHeader {
    pub fn to_string(&self) -> String {
        String::from_utf8(self.header.to_vec()).unwrap_or_else(|_| format!("{:?}", self.header))
    }
}

impl From<&[u8; 4]> for MessageHeader {
    fn from(value: &[u8; 4]) -> Self {
        return Self {
            header: value.clone(),
        };
    }
}

impl From<&[u8]> for MessageHeader {
    fn from(value: &[u8]) -> Self {
        let mut bytes = [0u8; 4];

        let len = value.len().min(4);
        bytes[..len].copy_from_slice(&value[..len]);

        return Self { header: bytes };
    }
}

impl From<&str> for MessageHeader {
    fn from(value: &str) -> Self {
        let mut bytes = [0u8; 4];

        let src = value.as_bytes();
        let len = src.len().min(4);
        bytes[..len].copy_from_slice(&src[..len]);

        return Self { header: bytes };
    }
}
