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
    pub name: Option<String>,
    pub background_color: Option<Color>,

    pub width: f32,
    pub height: f32,
    pub inner_walls: Vec<Rect>,

    pub enemy_groups: Vec<EnemyGroup>,
}

impl AreaTemplate {
    pub fn new(
        name: Option<String>,
        background_color: Option<Color>,
        width: f32,
        height: f32,
        inner_walls: Vec<Rect>,
        enemy_groups: Vec<EnemyGroup>,
    ) -> Self {
        Self {
            name,
            background_color,
            width,
            height,
            inner_walls,
            enemy_groups,
        }
    }
}

pub struct MapTemplate {
    pub id: String,
    pub name: String,
    pub background_color: Color,

    pub areas: Vec<AreaTemplate>,
}

impl MapTemplate {
    pub fn new(
        id: String,
        name: String,
        background_color: Color,
        mut areas: Vec<AreaTemplate>,
    ) -> Self {
        for (index, area) in areas.iter_mut().enumerate() {
            let area_name = area.name.clone().unwrap_or(format!("Area {}", index + 1));

            area.name = Some(format!("{} - {}", name, area_name));

            if area.background_color.is_none() {
                area.background_color = Some(background_color.clone());
            }
        }

        Self {
            id,
            name,
            background_color,
            areas,
        }
    }

    pub fn get_area(&self, id: usize) -> Option<&AreaTemplate> {
        self.areas.get(id)
    }
}
