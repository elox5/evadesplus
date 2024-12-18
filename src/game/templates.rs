use super::components::Color;
use crate::physics::rect::Rect;

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

pub struct AreaTemplate {
    pub id: String,
    pub name: String,
    pub background_color: Color,

    pub width: f32,
    pub height: f32,
    pub inner_walls: Vec<Rect>,

    pub enemy_groups: Vec<EnemyGroup>,
}

impl AreaTemplate {
    pub fn new(
        id: String,
        name: String,
        background_color: Color,
        width: f32,
        height: f32,
        inner_walls: Vec<Rect>,
        enemy_groups: Vec<EnemyGroup>,
    ) -> Self {
        Self {
            id,
            name,
            background_color,
            width,
            height,
            inner_walls,
            enemy_groups,
        }
    }
}
