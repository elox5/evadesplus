use hecs::With;

use crate::{
    game::components::{Direction, Position, Speed, Velocity},
    networking::rendering::{RenderNode, RenderPacket},
};

use super::{
    area::Area,
    components::{BounceOffBounds, Bounded, Color, Enemy, Player, Size},
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

pub fn system_bounds_check(area: &mut Area) {
    for (_, (dir, pos, size)) in area
        .world
        .query_mut::<With<(&mut Direction, &Position, &Size), &BounceOffBounds>>()
    {
        let bounds = &area.bounds;

        if (pos.0.x + size.0 / 2.0) > bounds.right() {
            dir.0.x *= -1.0;
        } else if (pos.0.x - size.0 / 2.0) < bounds.left() {
            dir.0.x *= -1.0;
        }
        if (pos.0.y + size.0 / 2.0) > bounds.bottom() {
            dir.0.y *= -1.0;
        } else if (pos.0.y - size.0 / 2.0) < bounds.top() {
            dir.0.y *= -1.0;
        }
    }

    for (_, (pos, size)) in area
        .world
        .query_mut::<With<(&mut Position, &Size), &Bounded>>()
    {
        let bounds = &area.bounds;

        // else if is allowed since the entity can only be outside the bounds in 1 direction
        // unless it's bigger than the area, in which case there's a bigger problem
        if (pos.0.x + size.0 / 2.0) > bounds.right() {
            pos.0.x = bounds.right() - size.0 / 2.0;
        } else if (pos.0.x - size.0 / 2.0) < bounds.left() {
            pos.0.x = bounds.left() + size.0 / 2.0;
        }
        if (pos.0.y + size.0 / 2.0) > bounds.bottom() {
            pos.0.y = bounds.bottom() - size.0 / 2.0;
        } else if (pos.0.y - size.0 / 2.0) < bounds.top() {
            pos.0.y = bounds.top() + size.0 / 2.0;
        }
    }
}

pub fn system_render(area: &mut Area) {
    area.render_packet = Some(RenderPacket::new());
    let nodes = &mut area.render_packet.as_mut().unwrap().nodes;

    for (_, (pos, size, color, player, enemy)) in
        area.world
            .query_mut::<(&Position, &Size, &Color, Option<&Player>, Option<&Enemy>)>()
    {
        let name = player.map(|p| p.name.clone());
        let node = RenderNode::new(
            pos.0.x,
            pos.0.y,
            size.0 / 2.0,
            color.clone(),
            enemy.is_some(),
            name,
        );
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
