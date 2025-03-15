use super::map::MapTemplate;
use crate::{config::CONFIG, logger::Logger, parsing::parse_map};
use std::{collections::HashMap, ffi::OsStr, sync::LazyLock};

static MAP_IDS: LazyLock<Vec<String>> = LazyLock::new(fill_map_ids);

static MAPS: LazyLock<HashMap<String, MapTemplate>> = LazyLock::new(fill_map_table);

fn fill_map_ids() -> Vec<String> {
    let config = &CONFIG.maps;

    let maps = match &config.maps.len() {
        0 => get_all_map_ids(&config.path),
        _ => config.maps.iter().map(String::from).collect(),
    };

    Logger::debug(format!("Loaded maps: {maps:?}"));

    maps
}

fn fill_map_table() -> HashMap<String, MapTemplate> {
    let path = &CONFIG.maps.path;

    let maps: HashMap<String, MapTemplate> = MAP_IDS
        .iter()
        .map(|id| {
            parse_map(&format!("{path}/{id}.yaml",))
                .unwrap_or_else(|err| panic!("Could not parse map {id}: {err}"))
        })
        .map(|map| {
            let template = MapTemplate::new(map);
            (template.id.clone(), template)
        })
        .collect();

    maps
}

fn get_all_map_ids(map_path: &str) -> Vec<String> {
    std::fs::read_dir(map_path)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter(|f| f.path().is_file())
        .filter(|f| f.path().extension().unwrap_or(OsStr::new("")) == "yaml")
        .map(|f| {
            f.file_name()
                .to_str()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>()
}

pub fn try_get_map(id: &str) -> Option<&MapTemplate> {
    MAPS.get(id)
}

pub fn get_map_table() -> &'static HashMap<String, MapTemplate> {
    &MAPS
}

pub fn get_map_list() -> Vec<&'static MapTemplate> {
    MAPS.values().collect()
}

pub fn map_exists(id: &str) -> bool {
    MAP_IDS.iter().any(|map_id| map_id == id)
}
