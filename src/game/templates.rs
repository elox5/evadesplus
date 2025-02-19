use std::collections::HashMap;

use super::{
    area::{AreaKey, Portal},
    components::Color,
};
use crate::physics::{rect::Rect, vec2::Vec2};

pub struct MapTemplate {
    pub id: String,
    pub name: String,
    pub background_color: Color,
    pub text_color: Color,

    pub areas: Vec<AreaTemplate>,

    alias_orders: HashMap<String, u16>,
}

impl MapTemplate {
    pub fn new(
        id: String,
        name: String,
        background_color: Color,
        text_color: Color,
        areas: Vec<AreaTemplate>,
    ) -> Self {
        let alias_orders: HashMap<String, u16> = areas
            .iter()
            .filter(|area| area.alias.is_some())
            .map(|area| (area.alias.clone().unwrap(), area.key.order()))
            .collect();

        Self {
            id,
            name,
            background_color,
            text_color,
            areas,
            alias_orders,
        }
    }

    pub fn try_get_area(&self, order: usize) -> Option<&AreaTemplate> {
        self.areas.get(order)
    }

    pub fn get_alias_order(&self, alias: &str) -> Option<u16> {
        self.alias_orders.get(alias).copied()
    }
}

pub struct AreaTemplate {
    pub key: AreaKey,
    pub alias: Option<String>,

    pub name: String,
    pub background_color: Color,

    pub width: f32,
    pub height: f32,

    pub spawn_pos: Vec2,

    pub inner_walls: Vec<Rect>,
    pub safe_zones: Vec<Rect>,
    pub portals: Vec<Portal>,

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
