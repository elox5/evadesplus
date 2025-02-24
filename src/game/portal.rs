use super::{area::AreaKey, components::Color, map_table::try_get_map};
use crate::physics::{rect::Rect, vec2::Vec2};
use serde::Deserialize;

#[derive(Clone)]
pub struct Portal {
    pub rect: Rect,
    pub color: Color,
    pub target: PortalTarget,
    pub target_pos: Vec2,
}

impl Portal {
    pub fn new(data: PortalData, ctx: &PortalCreationContext) -> Self {
        let target = match data.target {
            PortalTargetData::Area(id) => PortalTarget::Area(
                AreaKey::from_map_order_string(&id)
                    .unwrap_or_else(|_| panic!("Invalid portal target id: {id}")),
            ),
            PortalTargetData::Map(id) => PortalTarget::Map(id),
            PortalTargetData::Previous => {
                PortalTarget::Area(AreaKey::new(ctx.map_id.clone(), ctx.area_order as u16 - 1))
            }
            PortalTargetData::Next => {
                PortalTarget::Area(AreaKey::new(ctx.map_id.clone(), ctx.area_order as u16 + 1))
            }
        };

        let color = match data.color {
            Some(color) => color,
            None => match target {
                PortalTarget::Area(_) => "#ffff0033".to_owned(),
                PortalTarget::Map(_) => "#77ffff55".to_owned(),
            },
        };

        Portal {
            rect: data.rect,
            color: color.into(),
            target,
            target_pos: data.target_pos,
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
    pub target_pos: Vec2,
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
    Area(AreaKey),
    Map(String),
}

impl PortalTarget {
    pub fn get_area_key(&self) -> Option<AreaKey> {
        match self {
            PortalTarget::Area(key) => Some(key.clone()),
            PortalTarget::Map(id) => try_get_map(id).map(|map| map.get_start_area().key.clone()),
        }
    }
}
