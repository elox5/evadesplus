use serde::Serialize;

use crate::{game::map::MapData, networking::commands::get_command_cache};

#[derive(Serialize, Clone)]
pub struct MapCache {
    id: String,
    name: String,
    background_color: String,
    text_color: String,
}

impl MapCache {
    pub fn new(map: &MapData) -> Self {
        Self {
            id: map.id.clone(),
            name: map.name.clone(),
            background_color: map.background_color.clone(),
            text_color: map.text_color.clone(),
        }
    }
}

#[derive(Serialize, Clone)]
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
    pub fn new(map_data: &Vec<MapData>) -> Self {
        let maps = map_data.iter().map(MapCache::new).collect();

        let commands = get_command_cache();

        Self { maps, commands }
    }
}
