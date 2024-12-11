use crate::{game::components::Color, physics::vec2::Vec2};

pub struct RenderPacket {
    pub nodes: Vec<RenderNode>,
}

impl RenderPacket {
    const HEADER_SIZE: u32 = 13;

    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn to_datagrams(&self, max_size: u32, offset: Vec2) -> Vec<Vec<u8>> {
        let mut nodes = self.nodes.clone();
        let mut datagrams = Vec::new();

        while nodes.len() > 0 {
            let mut datagram = Vec::new();

            let clear: u8 = 0;

            datagram.extend_from_slice(&offset.x.to_le_bytes());
            datagram.extend_from_slice(&offset.y.to_le_bytes());
            datagram.push(clear);

            let mut datagram_nodes: Vec<RenderNode> = Vec::new();
            let mut node_total_size = 0;

            while node_total_size < max_size && nodes.len() > 0 {
                let node = nodes.pop().unwrap();
                let node_size = node.to_bytes().len() as u32;

                println!("Node size: {node_size} bytes. Remaining: {}", nodes.len());

                if Self::HEADER_SIZE + node_total_size + node_size > max_size {
                    nodes.push(node);
                    break;
                }

                node_total_size += node_size;
                datagram_nodes.push(node);
            }

            let node_count = datagram_nodes.len() as u32;

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
        // the last datagram has to tell the client
        // to render the frame

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
            let length = name.len();
            let capped_length = std::cmp::min(length, 255);

            bytes.push(capped_length as u8);
            bytes.extend_from_slice(&name.as_bytes()[..capped_length]);
        } else {
            bytes.push(0u8);
        }
        bytes
    }
}
