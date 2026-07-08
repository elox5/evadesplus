use itertools::Itertools;

use super::map::MapTemplate;
use crate::{config::CONFIG, game::map::MapData, logger::Logger, parsing::parse_map};
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

    let map_datas: Vec<MapData> = MAP_IDS
        .iter()
        .map(|id| {
            parse_map(&format!("{path}/{id}.yaml",))
                .unwrap_or_else(|err| panic!("Could not parse map {id}: {err}"))
        })
        .collect();

    let duplicate_groups = verify_no_duplicates(&map_datas);

    if !duplicate_groups.is_empty() {
        let list: Vec<String> = duplicate_groups
            .iter()
            .map(|(key, group)| {
                let names: Vec<String> = group.iter().map(|d| d.name.clone()).collect();
                format!("(key: {key}, maps: {names:?})")
            })
            .collect();

        let msg = format!("Map ID collision detected. Two maps can't share the same ID. {list:?}");

        Logger::error(msg.clone());
    }

    let maps: HashMap<String, MapTemplate> = map_datas
        .into_iter()
        .unique_by(|d| d.id.clone())
        .into_iter()
        .map(|map| {
            let template = MapTemplate::new(map);
            (template.id.clone(), template)
        })
        .collect();

    maps
}

fn verify_no_duplicates(map_datas: &Vec<MapData>) -> HashMap<String, Vec<&MapData>> {
    let mut id_groups: HashMap<String, Vec<&MapData>> = HashMap::new();
    for data in map_datas {
        let id = data.id.clone();
        id_groups.entry(id).or_insert_with(|| Vec::new()).push(data);
    }

    let duplicate_groups: HashMap<String, Vec<&MapData>> = id_groups
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .collect();

    duplicate_groups
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
