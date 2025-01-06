use super::{
    area::Portal,
    templates::{AreaTemplate, EnemyGroup, MapTemplate},
};
use crate::physics::{rect::Rect, vec2::Vec2};
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

#[derive(Deserialize)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub background_color: String,

    pub areas: Vec<AreaData>,
}

impl MapData {
    pub fn to_template(self) -> MapTemplate {
        let areas = self
            .areas
            .into_iter()
            .enumerate()
            .map(|(order, data)| {
                let area_id = data.id.unwrap_or(order.to_string());

                let background_color = data
                    .background_color
                    .unwrap_or(self.background_color.clone())
                    .into();

                let name = data.name.unwrap_or(format!("Area {}", order + 1));

                let portals = data.portals.unwrap_or_default();
                let portals = portals
                    .into_iter()
                    .map(|data| Portal {
                        rect: data.rect,
                        color: data.color.into(),
                        target_id: data.target_id,
                        target_pos: data.target_pos,
                    })
                    .collect::<Vec<_>>();

                let enemy_groups = data.enemy_groups.unwrap_or_default();
                let enemy_groups = enemy_groups
                    .into_iter()
                    .map(|data| EnemyGroup {
                        color: data.color.into(),
                        count: data.count,
                        speed: data.speed,
                        size: data.size,
                    })
                    .collect::<Vec<_>>();

                AreaTemplate {
                    order: order as u16,
                    area_id,
                    map_id: self.id.clone(),
                    area_name: name,
                    map_name: self.name.clone(),
                    background_color,
                    width: data.width,
                    height: data.height,
                    portals,
                    inner_walls: data.inner_walls.unwrap_or_default(),
                    safe_zones: data.safe_zones.unwrap_or_default(),
                    enemy_groups,
                }
            })
            .collect::<Vec<_>>();

        MapTemplate {
            id: self.id,
            name: self.name,
            background_color: self.background_color.into(),
            areas,
        }
    }
}

#[serde_inline_default]
#[derive(Deserialize)]
pub struct AreaData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub background_color: Option<String>,

    #[serde_inline_default(100.0)]
    pub width: f32,

    #[serde_inline_default(15.0)]
    pub height: f32,

    pub inner_walls: Option<Vec<Rect>>,
    pub safe_zones: Option<Vec<Rect>>,
    pub portals: Option<Vec<PortalData>>,

    pub enemy_groups: Option<Vec<EnemyGroupData>>,
}

#[derive(Deserialize)]
pub struct EnemyGroupData {
    pub color: String,
    pub count: u32,
    pub speed: f32,
    pub size: f32,
}

#[serde_inline_default]
#[derive(Deserialize)]
pub struct PortalData {
    pub rect: Rect,
    #[serde_inline_default("#ffff0033".to_owned())]
    pub color: String,
    pub target_id: String,
    pub target_pos: Vec2,
}
