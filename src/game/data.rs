use super::{
    area::{AreaKey, Portal},
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
                let background_color = data
                    .background_color
                    .unwrap_or(self.background_color.clone())
                    .into();

                let name = data.name.unwrap_or_else(|| format!("Area {}", order + 1));

                let portals = data.portals.unwrap_or_default();
                let portals = portals
                    .into_iter()
                    .map(|data| {
                        let target_key = match data.destination {
                            PortalDestination::Id(id) => AreaKey::from_map_order_string(&id)
                                .unwrap_or_else(|_| panic!("Invalid portal target id: {id}")),
                            PortalDestination::Previous => {
                                AreaKey::new(self.id.clone(), order as u16 - 1)
                            }
                            PortalDestination::Next => {
                                AreaKey::new(self.id.clone(), order as u16 + 1)
                            }
                        };

                        Portal {
                            rect: data.rect,
                            color: data.color.unwrap_or("#ffff0033".to_owned()).into(),
                            target_key,
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

                let key = AreaKey::new(self.id.clone(), order as u16);

                AreaTemplate {
                    key,
                    alias: data.alias,
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

        MapTemplate::new(
            self.id,
            self.name,
            self.background_color.into(),
            self.text_color.into(),
            areas,
        )
    }
}

#[derive(Deserialize)]
pub struct AreaData {
    pub alias: Option<String>,
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
