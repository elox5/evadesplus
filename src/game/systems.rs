use crate::{
    game::components::{Direction, Position, Speed, Velocity},
    networking::rendering::{RenderNode, RenderPacket},
};

use super::{
    area::Area,
    components::{Color, Player, Size},
};

pub fn system_update_position(area: &mut Area) {
    for (_, (pos, vel)) in area.world.query_mut::<(&mut Position, &Velocity)>() {
        pos.0 += vel.0 * area.delta_time;
    }
}

pub fn system_update_velocity(area: &mut Area) {
    for (_, (vel, dir, speed)) in area
        .world
        .query_mut::<(&mut Velocity, &Direction, &Speed)>()
    {
        vel.0 = dir.0 * speed.0;
    }
}

pub fn system_render(area: &mut Area) {
    area.render_packet = Some(RenderPacket::new());
    let nodes = &mut area.render_packet.as_mut().unwrap().nodes;

    for (_, (pos, size, color, player)) in area
        .world
        .query_mut::<(&Position, &Size, &Color, Option<&Player>)>()
    {
        let name = player.map(|p| p.name.clone());
        let node = RenderNode::new(pos.0.x, pos.0.y, size.0 / 2.0, color.clone(), false, name);
        nodes.push(node);
    }
}

pub fn system_send_render_packet(area: &mut Area) {
    if let Some(packet) = &area.render_packet {
        for (_, (player, pos)) in area.world.query_mut::<(&Player, &Position)>() {
            let _ = player.connection.send_datagram(packet.to_bytes(pos.0));
        }
    }
}
