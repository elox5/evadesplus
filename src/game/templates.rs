use super::components::Color;
use crate::physics::rect::Rect;

pub struct MapTemplate {
    pub id: String,
    pub name: String,
    pub background_color: Color,

    pub areas: Vec<AreaTemplate>,
}

impl MapTemplate {
    pub fn get_area(&self, id: &str) -> Option<&AreaTemplate> {
        self.areas.iter().find(|area| area.area_id == id)
    }
}

pub struct AreaTemplate {
    pub area_id: String,
    pub full_id: String,
    pub name: String,
    pub background_color: Color,

    pub width: f32,
    pub height: f32,
    pub inner_walls: Vec<Rect>,

    pub enemy_groups: Vec<EnemyGroup>,
}

#[derive(Clone)]
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
