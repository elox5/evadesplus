use crate::game::area::Area;

#[derive(Clone, Debug)]
pub struct AreaInfo {
    map_id: String,
    name: String,
    order: u16,
    color: Option<String>,
    victory: bool,
}

impl AreaInfo {
    pub fn new(area: &Area) -> Self {
        Self {
            map_id: area.key.map_id().to_owned(),
            name: area.name.clone(),
            order: area.key.order(),
            color: area.text_color.clone().map(|c| c.to_hex()),
            victory: area.flags.victory,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.map_id.len() as u8);
        bytes.extend_from_slice(self.map_id.as_bytes());
        bytes.push(self.name.len() as u8);
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.extend_from_slice(&self.order.to_le_bytes());
        bytes.push(self.victory as u8);

        if let Some(color) = &self.color {
            bytes.push(1);
            bytes.extend_from_slice(color.as_bytes());
        } else {
            bytes.push(0);
        }

        bytes
    }
}

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
        area_info: AreaInfo,
    },
    Remove,
    Transfer(AreaInfo),
    SetDowned(bool),
}

impl LeaderboardUpdate {
    pub fn add(player_id: u64, player_name: String, downed: bool, area_info: AreaInfo) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::Add {
                player_name,
                downed,
                area_info,
            },
        }
    }

    pub fn remove(player_id: u64) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::Remove,
        }
    }

    pub fn transfer(player_id: u64, area_info: AreaInfo) -> Self {
        Self {
            player_id,
            mode: LeaderboardUpdateMode::Transfer(area_info),
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
                area_info,
            } => {
                bytes.push(player_name.len().to_le_bytes()[0]); // 1 byte
                bytes.extend_from_slice(player_name.as_bytes()); // player_name.len() bytes
                bytes.push(*downed as u8); // 1 byte

                bytes.extend_from_slice(&area_info.to_bytes());
            }
            LeaderboardUpdateMode::Remove => {}
            LeaderboardUpdateMode::Transfer(area_info) => {
                bytes.extend_from_slice(&area_info.to_bytes());
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
    area_info: AreaInfo,
    downed: bool,
}

impl LeaderboardStateEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.player_id.to_le_bytes()); // 8 bytes
        bytes.push(self.player_name.len() as u8); // 1 byte
        bytes.extend_from_slice(self.player_name.as_bytes()); // player_name.len() bytes
        bytes.push(self.downed as u8); // 1 byte

        bytes.extend_from_slice(&self.area_info.to_bytes());

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
                area_info,
            } => self.add(LeaderboardStateEntry {
                player_id: update.player_id,
                player_name,
                downed,
                area_info,
            }),
            LeaderboardUpdateMode::Transfer(area_info) => {
                self.transfer(update.player_id, area_info);
            }
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

    fn transfer(&mut self, player_id: u64, area_info: AreaInfo) {
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
            area_info,
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
