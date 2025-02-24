use super::{area::AreaKey, components::Color, map_table::try_get_map};
use crate::physics::rect::Rect;
use anyhow::Result;
use serde::Deserialize;

#[derive(Clone)]
pub struct Portal {
    pub rect: Rect,
    pub color: Color,
    pub target: PortalTarget,
    pub target_x: PortalTargetPosX,
    pub target_y: PortalTargetPosY,
}

impl Portal {
    pub fn new(data: PortalData, ctx: &PortalCreationContext) -> Self {
        let target = match data.target {
            PortalTargetData::Area(id) => {
                let key = AreaKey::from_map_order_string(&id);

                match key {
                    Ok(key) => PortalTarget::AreaKey(key),
                    Err(_) => PortalTarget::AreaAlias(id),
                }
            }
            PortalTargetData::Map(id) => PortalTarget::Map(id),
            PortalTargetData::Previous => {
                PortalTarget::AreaKey(AreaKey::new(ctx.map_id.clone(), ctx.area_order as u16 - 1))
            }
            PortalTargetData::Next => {
                PortalTarget::AreaKey(AreaKey::new(ctx.map_id.clone(), ctx.area_order as u16 + 1))
            }
        };

        let color = match data.color {
            Some(color) => color,
            None => match target {
                PortalTarget::Map(_) => "#77ffff55".to_owned(),
                _ => "#ffff0033".to_owned(),
            },
        };

        Portal {
            rect: data.rect,
            color: color.into(),
            target,
            target_x: data.target_x,
            target_y: data.target_y,
        }
    }
}

pub struct PortalCreationContext {
    pub map_id: String,
    pub area_order: u16,
}

#[derive(Deserialize)]
pub struct PortalData {
    pub rect: Rect,
    pub color: Option<String>,
    pub target: PortalTargetData,
    pub target_x: PortalTargetPosX,
    pub target_y: PortalTargetPosY,
}

#[derive(Deserialize)]
pub enum PortalTargetData {
    Area(String),
    Map(String),
    Previous,
    Next,
}

#[derive(Clone)]
pub enum PortalTarget {
    AreaKey(AreaKey),
    AreaAlias(String),
    Map(String),
}

#[derive(Deserialize, Clone)]
pub enum PortalTargetPosX {
    FromLeft(f32),
    FromRight(f32),
    KeepPlayer,
    Center,
}

#[derive(Deserialize, Clone)]
pub enum PortalTargetPosY {
    FromBottom(f32),
    FromTop(f32),
    KeepPlayer,
    Center,
}

impl PortalTarget {
    pub fn get_area_key(&self) -> Result<AreaKey> {
        match self {
            PortalTarget::AreaKey(key) => Ok(key.clone()),
            PortalTarget::AreaAlias(id) => {
                let (map_id, alias) = id
                    .split_once(':')
                    .ok_or(anyhow::anyhow!("Could not parse portal target {id}"))?;

                match try_get_map(&map_id) {
                    Some(map) => Ok(map
                        .try_get_area_by_alias(alias)
                        .ok_or(anyhow::anyhow!(
                            "Could not find area with alias {alias} in map {map_id}"
                        ))?
                        .key
                        .clone()),
                    None => Err(anyhow::anyhow!("Map {map_id} not found")),
                }
            }
            PortalTarget::Map(id) => match try_get_map(id) {
                Some(map) => Ok(map.get_start_area().key.clone()),
                None => Err(anyhow::anyhow!("Map {id} not found")),
            },
        }
    }
}
