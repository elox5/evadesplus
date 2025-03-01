use std::hash::{DefaultHasher, Hash, Hasher};

use serde::Serialize;

use crate::{game::map::MapTemplate, networking::commands::get_command_cache};

#[derive(Serialize, Clone, Hash)]
pub struct MapCache {
    id: String,
    name: String,
    background_color: String,
    text_color: String,
}

impl MapCache {
    pub fn new(map: &MapTemplate) -> Self {
        Self {
            id: map.id.clone(),
            name: map.name.clone(),
            background_color: map.background_color.to_hex(),
            text_color: map.text_color.to_hex(),
        }
    }
}

#[derive(Serialize, Clone, Hash)]
pub struct CommandCache {
    name: String,
    description: String,
    usage: Option<String>,
    aliases: Option<Vec<String>>,
}

impl CommandCache {
    pub fn new(
        name: String,
        description: String,
        usage: Option<String>,
        aliases: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            description,
            usage,
            aliases,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Cache {
    maps: Vec<MapCache>,
    commands: Vec<CommandCache>,
}

impl Cache {
    pub fn new(map_data: Vec<&MapTemplate>) -> Self {
        let maps = map_data.into_iter().map(MapCache::new).collect();

        let commands = get_command_cache();

        Self { maps, commands }
    }

    pub fn get_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();

        self.maps.hash(&mut hasher);
        self.commands.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}
