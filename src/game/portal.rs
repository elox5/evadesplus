use super::{area::AreaKey, components::Color};
use crate::physics::{rect::Rect, vec2::Vec2};
use serde::Deserialize;

#[derive(Clone)]
pub struct Portal {
    pub rect: Rect,
    pub color: Color,
    pub target_key: AreaKey,
    pub target_pos: Vec2,
}

impl Portal {
    pub fn new(data: PortalData, ctx: &PortalCreationContext) -> Self {
        let target_key = match data.destination {
            PortalDestination::Id(id) => AreaKey::from_map_order_string(&id)
                .unwrap_or_else(|_| panic!("Invalid portal target id: {id}")),
            PortalDestination::Previous => {
                AreaKey::new(ctx.map_id.clone(), ctx.area_order as u16 - 1)
            }
            PortalDestination::Next => AreaKey::new(ctx.map_id.clone(), ctx.area_order as u16 + 1),
        };

        Portal {
            rect: data.rect,
            color: data.color.unwrap_or("#ffff0033".to_owned()).into(),
            target_key,
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
    pub destination: PortalDestination,
    pub target_pos: Vec2,
}

#[derive(Deserialize)]
pub enum PortalDestination {
    Id(String),
    Previous,
    Next,
}
