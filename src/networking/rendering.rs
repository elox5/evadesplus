use std::collections::HashMap;

use hecs::Entity;

use crate::{
    game::{area::AreaKey, components::Color, player::PlayerId},
    networking::new::user_registry::UserId,
};

#[derive(Clone)]
pub struct AreaRenderMessage {
    pub key: AreaKey,
    pub packet: AreaRenderPacket,
}

impl AreaRenderMessage {
    pub fn enrich(self, map: HashMap<PlayerId, UserId>) -> AreaRenderMessage {
        let nodes = self
            .packet
            .nodes
            .into_iter()
            .map(|n| {
                if n.is_hero
                    && let Some(entity) = n.entity
                {
                    let player_id = PlayerId {
                        entity,
                        area: self.key.clone(),
                    };

                    RenderNode {
                        x: n.x,
                        y: n.y,
                        radius: n.radius,
                        color: n.color,
                        has_border: n.has_border,
                        is_hero: n.is_hero,
                        downed: n.downed,
                        entity: n.entity,
                        user_id: map.get(&player_id).cloned(),
                        energy: n.energy,
                    }
                } else {
                    n
                }
            })
            .collect();

        return AreaRenderMessage {
            key: self.key,
            packet: AreaRenderPacket { nodes },
        };
    }
}

#[derive(Clone)]
pub struct AreaRenderPacket {
    pub nodes: Vec<RenderNode>,
}

impl AreaRenderPacket {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&(self.nodes.len() as u16).to_le_bytes());

        for node in &self.nodes {
            bytes.extend_from_slice(&node.to_bytes());
        }

        bytes
    }
}

#[derive(Clone)]
pub struct RenderNode {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: Color,
    pub has_border: bool,
    pub is_hero: bool,
    pub downed: bool,
    pub entity: Option<Entity>,
    pub user_id: Option<UserId>,
    pub energy: Option<f32>,
}

impl RenderNode {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.x.to_le_bytes());
        bytes.extend_from_slice(&self.y.to_le_bytes());
        bytes.extend_from_slice(&self.radius.to_le_bytes());
        bytes.extend_from_slice(&self.color.to_bytes());

        let has_energy = self.energy.is_some();

        let flags = (self.has_border as u8)
            | (self.is_hero as u8) << 1
            | (self.downed as u8) << 2
            | (has_energy as u8) << 3;

        bytes.push(flags);

        if self.is_hero
            && let Some(id) = &self.user_id
        {
            bytes.extend_from_slice(&id.0.to_le_bytes());
        }

        if let Some(energy) = self.energy {
            bytes.extend_from_slice(&energy.to_le_bytes());
        }

        bytes
    }

    pub fn length(&self) -> u32 {
        // x: 4 bytes
        // y: 4 bytes
        // radius: 4 bytes
        // color: 4 bytes
        // flags: 1 byte
        // entity: 8 bytes

        let length = 4 + 4 + 4 + 4 + 1 + 8;

        length
    }
}

#[derive(Clone)]
pub struct AreaDefinitionMessage {
    pub id: PlayerId,
    pub data: Vec<u8>,
}
