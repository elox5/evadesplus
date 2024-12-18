use super::{components::Color, data::EnemyGroup};
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
