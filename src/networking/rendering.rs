use crate::{game::components::Color, physics::vec2::Vec2};

pub struct RenderPacket {
    pub nodes: Vec<RenderNode>,
}

impl RenderPacket {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn to_bytes(&self, offset: Vec2) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&offset.x.to_le_bytes());
        bytes.extend_from_slice(&offset.y.to_le_bytes());
        bytes.push(self.nodes.len() as u8);
        for node in &self.nodes {
            bytes.extend_from_slice(&node.to_bytes());
        }
        bytes
    }
}

pub struct RenderNode {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: Color,
    pub has_border: bool,
    pub name: Option<String>,
}

impl RenderNode {
    pub fn new(
        x: f32,
        y: f32,
        radius: f32,
        color: Color,
        has_border: bool,
        name: Option<String>,
    ) -> RenderNode {
        RenderNode {
            x,
            y,
            radius,
            color,
            has_border,
            name,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.x.to_le_bytes());
        bytes.extend_from_slice(&self.y.to_le_bytes());
        bytes.extend_from_slice(&self.radius.to_le_bytes());
        bytes.extend_from_slice(&self.color.to_le_bytes());
        bytes.push(self.has_border as u8);
        if let Some(name) = &self.name {
            bytes.extend_from_slice(&name.len().to_le_bytes()[..4]);
            bytes.extend_from_slice(name.as_bytes());
        } else {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        bytes
    }
}
