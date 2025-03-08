use super::map::MapTemplate;
use crate::{
    env::{get_env_or_default, try_get_env_var},
    parsing::parse_map,
};
use std::{collections::HashMap, ffi::OsStr, sync::LazyLock};

static MAP_IDS: LazyLock<Vec<String>> = LazyLock::new(fill_map_ids);

static MAPS: LazyLock<HashMap<String, MapTemplate>> = LazyLock::new(fill_map_table);

fn fill_map_ids() -> Vec<String> {
    let map_path = get_env_or_default("MAP_PATH", "maps");
    let maps = try_get_env_var("MAPS");

    let maps = match maps {
        Some(maps) => maps.split(',').into_iter().map(str::to_owned).collect(),
        None => get_all_map_ids(map_path),
    };

    println!("Maps: {maps:?}");

    maps
}

fn fill_map_table() -> HashMap<String, MapTemplate> {
    let maps: HashMap<String, MapTemplate> = MAP_IDS
        .iter()
        .map(|id| {
            parse_map(&format!(
                "{}/{}.yaml",
                get_env_or_default("MAP_PATH", "maps"),
                id
            ))
            .unwrap_or_else(|err| panic!("Could not parse map {id}: {err}"))
        })
        .map(|map| {
            let template = MapTemplate::new(map);
            (template.id.clone(), template)
        })
        .collect();

    maps
}

fn get_all_map_ids(map_path: String) -> Vec<String> {
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
