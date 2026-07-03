use std::fmt::Display;

use hecs::Entity;

use crate::game::area::AreaKey;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId {
    pub entity: Entity,
    pub area: AreaKey,
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.entity.id(), self.area)
    }
}
