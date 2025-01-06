#[derive(Clone)]
pub struct ChatRequest {
    pub message: String,
    pub name: String,
}

impl ChatRequest {
    pub fn new(message: String, name: String) -> Self {
        Self { message, name }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"CHBR"); // 4 bytes (chat broadcast)
        bytes.push(self.name.len() as u8); // 1 byte
        bytes.push(self.message.len() as u8); // 1 byte
        bytes.extend_from_slice(self.name.as_bytes()); // name.len() bytes
        bytes.extend_from_slice(self.message.as_bytes()); // message.len() bytes

        bytes
    }
}
