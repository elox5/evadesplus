use super::{
    components::Color,
    templates::{AreaTemplate, MapTemplate},
};
use crate::physics::rect::Rect;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub background_color: Color,

    pub areas: Vec<AreaData>,
}

impl MapData {
    pub fn to_template(self) -> MapTemplate {
        let areas = self
            .areas
            .into_iter()
            .enumerate()
            .map(|(index, data)| {
                let area_id = data.id.unwrap_or(format!("{}", index));
                let full_id = format!("{}:{}", self.id.clone(), area_id.clone());
                let background_color = data
                    .background_color
                    .unwrap_or(self.background_color.clone());

                let name = data.name.unwrap_or(format!("Area {}", index + 1));
                let name = format!("{} - {}", self.name, name);

                AreaTemplate {
                    area_id,
                    full_id,
                    name,
                    background_color,
                    width: data.width,
                    height: data.height,
                    inner_walls: data.inner_walls.unwrap_or_default(),
                    enemy_groups: data.enemy_groups,
                }
            })
            .collect::<Vec<_>>();

        MapTemplate {
            id: self.id,
            name: self.name,
            background_color: self.background_color,
            areas,
        }
    }
}

#[derive(Deserialize)]
pub struct AreaData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub background_color: Option<Color>,

    pub width: f32,
    pub height: f32,

    pub inner_walls: Option<Vec<Rect>>,
    pub enemy_groups: Vec<EnemyGroup>,
}

#[derive(Clone, Deserialize)]
pub struct EnemyGroup {
    pub color: Color,
    pub count: u32,
    pub speed: f32,
    pub size: f32,
}

impl EnemyGroup {
    pub fn new(color: Color, count: u32, speed: f32, size: f32) -> Self {
        Self {
            color,
            count,
            speed,
            size,
        }
    }
}
