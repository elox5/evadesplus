use hecs::Entity;

use crate::{game::components::Color, physics::vec2::Vec2};

pub struct RenderPacket {
    pub nodes: Vec<RenderNode>,
}

impl RenderPacket {
    const HEADER_SIZE: u32 = 11;

    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn to_datagrams(&self, max_size: u32, offset: Vec2) -> Vec<Vec<u8>> {
        let mut nodes = self.nodes.clone();
        let mut datagrams = Vec::new();

        while !nodes.is_empty() {
            let mut datagram = Vec::new();

            let should_render: u8 = 0;

            datagram.extend_from_slice(&offset.x.to_le_bytes());
            datagram.extend_from_slice(&offset.y.to_le_bytes());
            datagram.push(should_render);

            let mut datagram_nodes: Vec<RenderNode> = Vec::new();
            let mut node_total_size = 0;

            while node_total_size < max_size && !nodes.is_empty() {
                let node = nodes.pop().unwrap();
                let node_size = node.length();

                // println!("Node size: {node_size} bytes. Remaining: {}", nodes.len());

                if Self::HEADER_SIZE + node_total_size + node_size > max_size {
                    nodes.push(node);
                    break;
                }

                node_total_size += node_size;
                datagram_nodes.push(node);
            }

            let node_count = datagram_nodes.len() as u16;

            datagram.extend_from_slice(&node_count.to_le_bytes());
            for node in &datagram_nodes {
                datagram.extend_from_slice(&node.to_bytes());
            }

            // println!(
            //     "Creating datagram. Size: {} bytes. Node count: {node_count}",
            //     datagram.len()
            // );

            datagrams.push(datagram);
        }

        let datagrams_len = datagrams.len();
        datagrams[datagrams_len - 1][8] = 1;
        // the last datagram has to tell the client to render the frame

        // println!("Packet ready. Datagrams sent: {}", datagrams.len());

        datagrams
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
    pub entity: Entity,
    pub player_id: Option<u64>,
}

impl RenderNode {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.x.to_le_bytes());
        bytes.extend_from_slice(&self.y.to_le_bytes());
        bytes.extend_from_slice(&self.radius.to_le_bytes());
        bytes.extend_from_slice(&self.color.to_bytes());

        let flags = (self.has_border as u8) | (self.is_hero as u8) << 1 | (self.downed as u8) << 2;

        bytes.push(flags);

        bytes.extend_from_slice(&self.player_id.unwrap_or(0u64).to_le_bytes());

        bytes
    }

    pub fn length(&self) -> u32 {
        // x: 4 bytes
        // y: 4 bytes
        // radius: 4 bytes
        // color: 4 bytes
        // flags: 1 byte
        // player_id: 8 bytes

        let length = 4 + 4 + 4 + 4 + 1 + 8;

        length
    }
}
