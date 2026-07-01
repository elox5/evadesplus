use hecs::Entity;

use crate::game::area::AreaKey;

pub struct Player {
    pub id: u64,
    pub name: String,
    pub entity: Entity,
    pub area_key: AreaKey,
    pub victories: Vec<AreaKey>,
}

impl Player {
    pub fn new(id: u64, name: String, entity: Entity, area_key: AreaKey) -> Self {
        Self {
            id,
            entity,
            area_key,
            name,
            victories: Vec::new(),
        }
    }
}
