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

    start_area_order: u16,

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

        let start_area_order = data.start_area_order.unwrap_or(0);

        if areas.get(start_area_order as usize).is_none() {
            panic!(
                "Could not find area with order {} in map {} to set as start area",
                start_area_order, data.id
            );
        }

        Self {
            id: data.id,
            name: data.name,
            background_color: data.background_color.into(),
            text_color: data.text_color.into(),
            areas,
            start_area_order,
            alias_orders,
        }
    }

    pub fn try_get_area(&self, order: usize) -> Option<&AreaTemplate> {
        self.areas.get(order)
    }

    pub fn try_get_area_by_alias(&self, alias: &str) -> Option<&AreaTemplate> {
        let order = *self.alias_orders.get(alias)?;
        self.areas.get(order as usize)
    }

    pub fn get_start_area(&self) -> &AreaTemplate {
        self.areas.get(self.start_area_order as usize).unwrap()
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

    pub start_area_order: Option<u16>,
}
