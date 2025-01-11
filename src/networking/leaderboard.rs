use hecs::Entity;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Debug)]
pub struct LeaderboardUpdate {
    hash: u64,
    pub mode: LeaderboardUpdateMode,
}

#[derive(Clone, Debug)]
pub enum LeaderboardUpdateMode {
    Add {
        player_name: String,
        downed: bool,
        area_order: u16,
        area_name: String,
        map_name: String,
    },
    Remove,
    Transfer {
        old_hash: u64,
        area_order: u16,
        area_name: String,
        map_name: String,
    },
    SetDowned(bool),
}

impl LeaderboardUpdate {
    pub fn add(
        entity: Entity,
        area_full_id: String,
        player_name: String,
        downed: bool,
        area_order: u16,
        area_name: String,
        map_name: String,
    ) -> Self {
        Self {
            hash: Self::get_hash(&entity, &area_full_id),
            mode: LeaderboardUpdateMode::Add {
                player_name,
                downed,
                area_order,
                area_name,
                map_name,
            },
        }
    }

    pub fn remove(entity: Entity, area_full_id: String) -> Self {
        Self {
            hash: Self::get_hash(&entity, &area_full_id),
            mode: LeaderboardUpdateMode::Remove,
        }
    }

    pub fn transfer(
        entity: Entity,
        old_entity: Entity,
        old_area_full_id: String,
        new_area_order: u16,
        new_area_full_id: String,
        new_area_name: String,
        new_map_name: String,
    ) -> Self {
        Self {
            hash: Self::get_hash(&entity, &new_area_full_id),
            mode: LeaderboardUpdateMode::Transfer {
                old_hash: Self::get_hash(&old_entity, &old_area_full_id),
                area_order: new_area_order,
                area_name: new_area_name,
                map_name: new_map_name,
            },
        }
    }

    pub fn set_downed(entity: Entity, area_full_id: String, downed: bool) -> Self {
        Self {
            hash: Self::get_hash(&entity, &area_full_id),
            mode: LeaderboardUpdateMode::SetDowned(downed),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let header = match &self.mode {
            LeaderboardUpdateMode::Add { .. } => "LBAD",
            LeaderboardUpdateMode::Remove => "LBRM",
            LeaderboardUpdateMode::Transfer { .. } => "LBTR",
            LeaderboardUpdateMode::SetDowned(_) => "LBSD",
        };

        bytes.extend_from_slice(header.as_bytes()); // 4 bytes
        bytes.extend_from_slice(&self.hash.to_le_bytes()); // 8 bytes

        match &self.mode {
            LeaderboardUpdateMode::Add {
                player_name,
                downed,
                area_order,
                area_name,
                map_name,
            } => {
                bytes.extend_from_slice(&area_order.to_le_bytes()); // 2 bytes
                bytes.push(*downed as u8); // 1 byte
                bytes.push(player_name.len().to_le_bytes()[0]); // 1 byte
                bytes.push(area_name.len().to_le_bytes()[0]); // 1 byte
                bytes.push(map_name.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(player_name.as_bytes()); // player_name.len() bytes
                bytes.extend_from_slice(area_name.as_bytes()); // area_name.len() bytes
                bytes.extend_from_slice(map_name.as_bytes()); // map_name.len() bytes
            }
            LeaderboardUpdateMode::Remove => {}
            LeaderboardUpdateMode::Transfer {
                old_hash,
                area_order,
                area_name,
                map_name,
            } => {
                bytes.extend_from_slice(&old_hash.to_le_bytes()); // 8 bytes
                bytes.extend_from_slice(&area_order.to_le_bytes()); // 2 bytes
                bytes.push(area_name.len().to_le_bytes()[0]); // 1 byte
                bytes.push(map_name.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(area_name.as_bytes()); // area_name.len() bytes
                bytes.extend_from_slice(map_name.as_bytes()); // map_name.len() bytes
            }
            LeaderboardUpdateMode::SetDowned(downed) => {
                bytes.push(*downed as u8); // 1 byte
            }
        }

        bytes
    }

    fn get_hash(entity: &Entity, area_full_id: &str) -> u64 {
        let mut hasher = DefaultHasher::new();

        entity.id().hash(&mut hasher);
        area_full_id.hash(&mut hasher);

        hasher.finish()
    }
}

#[derive(Clone)]
struct LeaderboardStateEntry {
    hash: u64,
    player_name: String,
    downed: bool,
    area_order: u16,
    area_name: String,
    map_name: String,
}

impl LeaderboardStateEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.hash.to_le_bytes()); // 8 bytes
        bytes.extend_from_slice(&self.area_order.to_le_bytes()); // 2 bytes
        bytes.push(self.downed as u8); // 1 byte
        bytes.push(self.player_name.len() as u8); // 1 byte
        bytes.push(self.area_name.len() as u8); // 1 byte
        bytes.push(self.map_name.len() as u8); // 1 byte
        bytes.extend_from_slice(self.player_name.as_bytes()); // player_name.len() bytes
        bytes.extend_from_slice(self.area_name.as_bytes()); // area_name.len() bytes
        bytes.extend_from_slice(self.map_name.as_bytes()); // map_name.len() bytes

        bytes
    }
}

#[derive(Clone)]
pub struct LeaderboardState {
    entries: Vec<LeaderboardStateEntry>,
}

impl LeaderboardState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn update(&mut self, update: LeaderboardUpdate) {
        match update.mode {
            LeaderboardUpdateMode::Add {
                player_name,
                downed,
                area_order,
                area_name,
                map_name,
            } => self.add(LeaderboardStateEntry {
                hash: update.hash,
                player_name,
                downed,
                area_order,
                area_name,
                map_name,
            }),
            LeaderboardUpdateMode::Transfer {
                old_hash,
                area_order,
                area_name,
                map_name,
            } => self.transfer(old_hash, update.hash, area_order, area_name, map_name),
            LeaderboardUpdateMode::Remove => self.remove(update.hash),
            LeaderboardUpdateMode::SetDowned(downed) => self.set_downed(update.hash, downed),
        }
    }

    fn add(&mut self, entry: LeaderboardStateEntry) {
        self.entries.push(entry);
    }

    fn remove(&mut self, hash: u64) {
        let index = self.entries.iter().position(|e| e.hash == hash).unwrap();

        self.entries.swap_remove(index);
    }

    fn transfer(
        &mut self,
        old_hash: u64,
        new_hash: u64,
        area_order: u16,
        area_name: String,
        map_name: String,
    ) {
        let old_entry_index = self
            .entries
            .iter()
            .position(|e| e.hash == old_hash)
            .unwrap();

        let old_entry = self.entries.swap_remove(old_entry_index);

        self.add(LeaderboardStateEntry {
            hash: new_hash,
            player_name: old_entry.player_name,
            downed: old_entry.downed,
            area_order,
            area_name,
            map_name,
        });
    }

    fn set_downed(&mut self, hash: u64, downed: bool) {
        for entry in &mut self.entries {
            if entry.hash == hash {
                entry.downed = downed;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"LBST");
        bytes.push(self.entries.len() as u8);

        for entry in &self.entries {
            bytes.extend_from_slice(&entry.to_bytes());
        }

        bytes
    }
}
