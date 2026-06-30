#[derive(Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub bytes: [u8; 4],
}

impl MessageHeader {
    pub fn to_string(&self) -> String {
        String::from_utf8(self.bytes.to_vec()).unwrap_or_else(|_| format!("{:?}", self.bytes))
    }
}

impl From<&[u8; 4]> for MessageHeader {
    fn from(value: &[u8; 4]) -> Self {
        return Self {
            bytes: value.clone(),
        };
    }
}

impl From<&[u8]> for MessageHeader {
    fn from(value: &[u8]) -> Self {
        let mut bytes = [0u8; 4];

        let len = value.len().min(4);
        bytes[..len].copy_from_slice(&value[..len]);

        return Self { bytes };
    }
}

impl From<&str> for MessageHeader {
    fn from(value: &str) -> Self {
        let mut bytes = [0u8; 4];

        let src = value.as_bytes();
        let len = src.len().min(4);
        bytes[..len].copy_from_slice(&src[..len]);

        return Self { bytes };
    }
}
