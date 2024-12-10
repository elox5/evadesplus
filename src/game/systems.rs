use super::{
    area::Area,
    components::{BounceOffBounds, Bounded, Color, Enemy, Hero, Player, Size},
};
use crate::{
    game::components::{Direction, Position, Speed, Velocity},
    networking::rendering::{RenderNode, RenderPacket},
};
use hecs::With;

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

pub fn system_enemy_collision(area: &mut Area) {
    for (_, (hero_pos, hero_size, hero_color)) in area
        .world
        .query::<With<(&Position, &Size, &mut Color), &Hero>>()
        .iter()
    {
        let hero_pos = hero_pos.0;
        let hero_size = hero_size.0;

        for (_, (enemy_pos, enemy_size)) in area
            .world
            .query::<With<(&Position, &Size), &Enemy>>()
            .iter()
        {
            let enemy_pos = enemy_pos.0;
            let enemy_size = enemy_size.0;

            let distance_sq = (hero_pos - enemy_pos).magnitude_sq();
            let radius_sum = (hero_size + enemy_size) * 0.5;

            if distance_sq < radius_sum * radius_sum {
                *hero_color = Color::rgb(rand::random(), rand::random(), rand::random());
            }
        }
    }
}

pub fn system_hero_collision(area: &mut Area) {
    for (entity_1, (pos_1, size_1)) in area.world.query::<With<(&Position, &Size), &Hero>>().iter()
    {
        for (entity_2, (pos_2, size_2)) in
            area.world.query::<With<(&Position, &Size), &Hero>>().iter()
        {
            if entity_1 == entity_2 {
                continue;
            }

            let distance_sq = (pos_1.0 - pos_2.0).magnitude_sq();
            let radius_sum = (size_1.0 + size_2.0) * 0.5;

            if distance_sq < radius_sum * radius_sum {
                println!("Collision: {:?} and {:?}", entity_1, entity_2);
            }
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
            if let Some(max_datagram_size) = player.connection.max_datagram_size() {
                let datagrams = packet.to_datagrams(max_datagram_size as u32, pos.0);

                for datagram in datagrams {
                    let _ = player.connection.send_datagram(datagram);
                }
            }
        }
    }
}
