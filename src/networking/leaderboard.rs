#[derive(Clone, Debug)]
pub struct LeaderboardUpdate {
    player_id: u64,
    pub mode: LeaderboardUpdateMode,
}

#[derive(Clone, Debug)]
pub enum LeaderboardUpdateMode {
    Add {
        player_name: String,
        downed: bool,
        area_order: u16,
        area_name: String,
        map_id: String,
    },
    Remove,
    Transfer {
        area_order: u16,
        area_name: String,
        map_id: String,
    },
    SetDowned(bool),
}

impl LeaderboardUpdate {
    pub fn add(
        player_id: u64,
        player_name: String,
        downed: bool,
        area_order: u16,
        area_name: String,
        map_id: String,
    ) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::Add {
                player_name,
                downed,
                area_order,
                area_name,
                map_id,
            },
        }
    }

    pub fn remove(player_id: u64) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::Remove,
        }
    }

    pub fn transfer(
        player_id: u64,
        target_area_order: u16,
        target_area_name: String,
        target_map_id: String,
    ) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::Transfer {
                area_order: target_area_order,
                area_name: target_area_name,
                map_id: target_map_id,
            },
        }
    }

    pub fn set_downed(player_id: u64, downed: bool) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::SetDowned(downed),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let header = match &self.mode {
            LeaderboardUpdateMode::Add { .. } => "PADD",
            LeaderboardUpdateMode::Remove => "PRMV",
            LeaderboardUpdateMode::Transfer { .. } => "PTRF",
            LeaderboardUpdateMode::SetDowned(_) => "PSDN",
        };

        bytes.extend_from_slice(header.as_bytes()); // 4 bytes
        bytes.extend_from_slice(&self.player_id.to_le_bytes()); // 8 bytes

        match &self.mode {
            LeaderboardUpdateMode::Add {
                player_name,
                downed,
                area_order,
                area_name,
                map_id,
            } => {
                bytes.extend_from_slice(&area_order.to_le_bytes()); // 2 bytes
                bytes.push(*downed as u8); // 1 byte
                bytes.push(player_name.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(player_name.as_bytes()); // player_name.len() bytes
                bytes.push(area_name.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(area_name.as_bytes()); // area_name.len() bytes
                bytes.push(map_id.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(map_id.as_bytes()); // map_id.len() bytes
            }
            LeaderboardUpdateMode::Remove => {}
            LeaderboardUpdateMode::Transfer {
                area_order,
                area_name,
                map_id,
            } => {
                bytes.extend_from_slice(&area_order.to_le_bytes()); // 2 bytes
                bytes.push(area_name.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(area_name.as_bytes()); // area_name.len() bytes
                bytes.push(map_id.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(map_id.as_bytes()); // map_name.len() bytes
            }
            LeaderboardUpdateMode::SetDowned(downed) => {
                bytes.push(*downed as u8); // 1 byte
            }
        }

        bytes
    }
}

#[derive(Clone)]
struct LeaderboardStateEntry {
    player_id: u64,
    player_name: String,
    downed: bool,
    area_order: u16,
    area_name: String,
    map_id: String,
}

impl LeaderboardStateEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.player_id.to_le_bytes()); // 8 bytes
        bytes.extend_from_slice(&self.area_order.to_le_bytes()); // 2 bytes
        bytes.push(self.downed as u8); // 1 byte
        bytes.push(self.player_name.len() as u8); // 1 byte
        bytes.extend_from_slice(self.player_name.as_bytes()); // player_name.len() bytes
        bytes.push(self.area_name.len() as u8); // 1 byte
        bytes.extend_from_slice(self.area_name.as_bytes()); // area_name.len() bytes
        bytes.push(self.map_id.len() as u8); // 1 byte
        bytes.extend_from_slice(self.map_id.as_bytes()); // map_name.len() bytes

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
                map_id,
            } => self.add(LeaderboardStateEntry {
                player_id: update.player_id,
                player_name,
                downed,
                area_order,
                area_name,
                map_id,
            }),
            LeaderboardUpdateMode::Transfer {
                area_order,
                area_name,
                map_id,
            } => self.transfer(update.player_id, area_order, area_name, map_id),
            LeaderboardUpdateMode::Remove => self.remove(update.player_id),
            LeaderboardUpdateMode::SetDowned(downed) => self.set_downed(update.player_id, downed),
        }
    }

    fn add(&mut self, entry: LeaderboardStateEntry) {
        self.entries.push(entry);
    }

    fn remove(&mut self, player_id: u64) {
        let index = self
            .entries
            .iter()
            .position(|e| e.player_id == player_id)
            .unwrap();

        self.entries.swap_remove(index);
    }

    fn transfer(&mut self, player_id: u64, area_order: u16, area_name: String, map_id: String) {
        let old_entry_index = self
            .entries
            .iter()
            .position(|e| e.player_id == player_id)
            .unwrap();

        let old_entry = self.entries.swap_remove(old_entry_index);

        self.add(LeaderboardStateEntry {
            player_id,
            player_name: old_entry.player_name,
            downed: old_entry.downed,
            area_order,
            area_name,
            map_id,
        });
    }

    fn set_downed(&mut self, player_id: u64, downed: bool) {
        for entry in &mut self.entries {
            if entry.player_id == player_id {
                entry.downed = downed;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.entries.len() as u8);

        for entry in &self.entries {
            bytes.extend_from_slice(&entry.to_bytes());
        }

        bytes
    }
}
