use super::map::{MapData, MapTemplate};
use crate::{
    env::{get_env_or_default, try_get_env_var},
    parsing::parse_map,
};
use std::{collections::HashMap, ffi::OsStr, sync::LazyLock};

static MAPS: LazyLock<HashMap<String, MapTemplate>> = LazyLock::new(fill_map_list);

fn fill_map_list() -> HashMap<String, MapTemplate> {
    let map_path = get_env_or_default("MAP_PATH", "maps");
    let maps = try_get_env_var("MAPS");

    let maps = match maps {
        Some(maps) => get_selected_maps(maps, map_path),
        None => get_all_maps(map_path),
    };

    let maps: HashMap<String, MapTemplate> = maps
        .into_iter()
        .map(|map| {
            let template = MapTemplate::new(map);
            (template.id.clone(), template)
        })
        .collect();

    maps
}

fn get_selected_maps(map_list: String, map_path: String) -> Vec<MapData> {
    map_list
        .split(',')
        .into_iter()
        .map(|m| parse_map(&format!("{}/{}.yaml", map_path, m)).unwrap())
        .collect::<Vec<_>>()
}

fn get_all_maps(map_path: String) -> Vec<MapData> {
    std::fs::read_dir(map_path)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter(|f| f.path().is_file())
        .filter(|f| f.path().extension().unwrap_or(OsStr::new("")) == "yaml")
        .map(|f| parse_map(f.path().to_str().unwrap()).unwrap())
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
