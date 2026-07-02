use crate::game::player::PlayerId;

#[derive(Clone)]
pub struct TimerSyncPacket {
    pub player_id: PlayerId,
    pub time: f32,
}

impl TimerSyncPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"TIME");
        bytes.extend_from_slice(&self.time.to_le_bytes());
        bytes
    }
}
