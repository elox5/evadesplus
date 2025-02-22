use crate::game::map::MapData;
use anyhow::Result;

pub fn parse_map(path: &str) -> Result<MapData> {
    let file = std::fs::read_to_string(path)?;

    let map: MapData = serde_yaml::from_str(&file)?;

    Ok(map)
}
