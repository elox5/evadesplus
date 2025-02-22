use super::{
    area::{AreaCreationContext, AreaData, AreaTemplate},
    components::Color,
};
use serde::Deserialize;
use std::collections::HashMap;

pub struct MapTemplate {
    pub id: String,
    pub name: String,
    pub background_color: Color,
    pub text_color: Color,

    pub areas: Vec<AreaTemplate>,

    alias_orders: HashMap<String, u16>,
}

impl MapTemplate {
    pub fn new(data: MapData) -> Self {
        let area_ctx = AreaCreationContext {
            map_id: data.id.clone(),
            background_color: data.background_color.clone(),
        };

        let areas: Vec<AreaTemplate> = data
            .areas
            .into_iter()
            .enumerate()
            .map(|(order, area)| AreaTemplate::new(area, order as u16, &area_ctx))
            .collect();

        let alias_orders: HashMap<String, u16> = areas
            .iter()
            .filter(|area| area.alias.is_some())
            .map(|area| (area.alias.clone().unwrap(), area.key.order()))
            .collect();

        Self {
            id: data.id,
            name: data.name,
            background_color: data.background_color.into(),
            text_color: data.text_color.into(),
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

#[derive(Deserialize)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub background_color: String,
    pub text_color: String,

    pub areas: Vec<AreaData>,
}
