use super::{
    area::Portal,
    templates::{AreaTemplate, EnemyGroup, MapTemplate},
};
use crate::physics::{rect::Rect, vec2::Vec2};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub background_color: String,
    pub text_color: String,

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

                let name = data.name.unwrap_or_else(|| format!("Area {}", order + 1));

                let portals = data.portals.unwrap_or_default();
                let portals = portals
                    .into_iter()
                    .map(|data| {
                        let target_id = match data.destination {
                            PortalDestination::Id(id) => id,
                            PortalDestination::Previous => format!("{}:{}", self.id, order - 1),
                            PortalDestination::Next => format!("{}:{}", self.id, order + 1),
                        };

                        Portal {
                            rect: data.rect,
                            color: data.color.unwrap_or("#ffff0033".to_owned()).into(),
                            target_id,
                            target_pos: data.target_pos,
                        }
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

                let width = data.width.unwrap_or(100.0);
                let height = data.height.unwrap_or(15.0);

                AreaTemplate {
                    order: order as u16,
                    area_id,
                    map_id: self.id.clone(),
                    name,
                    background_color,
                    width,
                    height,
                    spawn_pos: data
                        .spawn_pos
                        .unwrap_or_else(|| Vec2::new(5.0, height / 2.0)),
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
            text_color: self.text_color.into(),
            areas,
        }
    }
}

#[derive(Deserialize)]
pub struct AreaData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub background_color: Option<String>,

    pub width: Option<f32>,
    pub height: Option<f32>,

    pub spawn_pos: Option<Vec2>,

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
