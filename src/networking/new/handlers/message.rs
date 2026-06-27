use crate::networking::new::client_message::{ClientMessage, MessageHeader};

pub trait Message {
    fn header(&self) -> MessageHeader;
    fn data(&self) -> Vec<u8>;
}

impl Message for ClientMessage {
    fn header(&self) -> MessageHeader {
        self.header.clone()
    }

    fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
