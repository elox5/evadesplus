use hecs::Entity;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone)]
pub struct LeaderboardEntry {
    player_name: String,
    map_name: String,
    area_name: String,
    area_order: u16,
}

impl LeaderboardEntry {
    pub fn new(player_name: String, map_name: String, area_name: String, area_order: u16) -> Self {
        Self {
            player_name,
            map_name,
            area_name,
            area_order,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.area_order.to_le_bytes()); // 2 bytes
        bytes.push(self.player_name.len().to_le_bytes()[0]); // 1 byte
        bytes.push(self.area_name.len().to_le_bytes()[0]); // 1 byte
        bytes.push(self.map_name.len().to_le_bytes()[0]); // 1 byte
        bytes.extend_from_slice(self.player_name.as_bytes()); // player_name.len() bytes
        bytes.extend_from_slice(self.area_name.as_bytes()); // area_name.len() bytes
        bytes.extend_from_slice(self.map_name.as_bytes()); // map_name.len() bytes

        bytes
    }

    pub fn length(&self) -> u8 {
        2 + 3 + (self.player_name.len() + self.area_name.len() + self.map_name.len()) as u8
    }
}

#[derive(Clone)]
pub struct LeaderboardUpdatePacket {
    entity: Entity,
    area_full_id: String,
    pub update: LeaderboardUpdate,
}

#[derive(Clone)]
pub enum LeaderboardUpdate {
    Add(LeaderboardEntry),
    Remove,
}

impl LeaderboardUpdatePacket {
    pub fn add(
        entity: Entity,
        area_full_id: String,
        player_name: String,
        map_name: String,
        area_name: String,
        area_order: u16,
    ) -> Self {
        Self {
            entity,
            area_full_id,
            update: LeaderboardUpdate::Add(LeaderboardEntry::new(
                player_name,
                map_name,
                area_name,
                area_order,
            )),
        }
    }

    pub fn remove(entity: Entity, area_full_id: String) -> Self {
        Self {
            entity,
            area_full_id,
            update: LeaderboardUpdate::Remove,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"LBUP");
        bytes.extend_from_slice(&self.to_bytes_no_header());

        bytes
    }

    pub fn to_bytes_no_header(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let is_add = match &self.update {
            LeaderboardUpdate::Add { .. } => true,
            LeaderboardUpdate::Remove => false,
        };

        bytes.push(is_add as u8); // 1 byte
        bytes.extend_from_slice(&self.get_hash().to_le_bytes()); // 8 bytes

        match &self.update {
            LeaderboardUpdate::Add(entry) => {
                bytes.extend_from_slice(&entry.to_bytes());
            }
            LeaderboardUpdate::Remove => {}
        }

        bytes
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.entity.id().hash(&mut hasher);
        self.area_full_id.hash(&mut hasher);

        hasher.finish()
    }
}

#[derive(Clone)]
pub struct LeaderboardState {
    entries: Vec<LeaderboardUpdatePacket>,
}

impl LeaderboardState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: LeaderboardUpdatePacket) {
        self.entries.push(entry);
    }

    pub fn remove(&mut self, hash: u64) {
        self.entries.retain(|e| e.get_hash() != hash);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"LBST");
        bytes.push(self.entries.len() as u8);

        for entry in &self.entries {
            let entry_bytes = &entry.to_bytes_no_header();
            let length = entry_bytes.len();

            bytes.push(length as u8);
            bytes.extend_from_slice(entry_bytes);
        }

        bytes
    }
}
