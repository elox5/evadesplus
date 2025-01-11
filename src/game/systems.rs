use super::{
    area::Area,
    components::{
        BounceOffBounds, Bounded, Color, Downed, Enemy, Hero, Named, RenderReceiver, Size,
    },
    game::TransferRequest,
};
use crate::{
    game::components::{Direction, Position, Speed, Velocity},
    networking::{
        leaderboard::LeaderboardUpdate,
        rendering::{RenderNode, RenderPacket},
    },
};
use hecs::{With, Without};

pub fn system_update_position(area: &mut Area) {
    for (_, (pos, vel)) in area
        .world
        .query_mut::<Without<(&mut Position, &Velocity), &Downed>>()
    {
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

        if (pos.0.x + size.0 / 2.0) > bounds.right() || (pos.0.x - size.0 / 2.0) < bounds.left() {
            dir.0.x *= -1.0;
        }

        if (pos.0.y + size.0 / 2.0) > bounds.bottom() || (pos.0.y - size.0 / 2.0) < bounds.top() {
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

pub fn system_inner_wall_collision(area: &mut Area) {
    if area.inner_walls.is_empty() {
        return;
    }

    for (_, (dir, pos, size)) in area
        .world
        .query_mut::<With<(&mut Direction, &Position, &Size), &BounceOffBounds>>()
    {
        for wall in &area.inner_walls {
            if wall.contains_circle(pos.0, size.0 / 2.0) {
                let distance = wall.center() - pos.0;

                let x_distance = distance.x / wall.w;
                let y_distance = distance.y / wall.h;

                if x_distance.abs() > y_distance.abs() {
                    dir.0.x *= -1.0;
                } else {
                    dir.0.y *= -1.0;
                }
            }
        }
    }

    for (_, (pos, size)) in area
        .world
        .query_mut::<With<(&mut Position, &Size), &Bounded>>()
    {
        for wall in &area.inner_walls {
            if wall.contains_circle(pos.0, size.0 / 2.0) {
                let distance = wall.center() - pos.0;

                let x_distance = distance.x / wall.w;
                let y_distance = distance.y / wall.h;

                if x_distance.abs() > y_distance.abs() {
                    if x_distance < 0.0 {
                        pos.0.x = wall.center().x + wall.w / 2.0 + size.0 / 2.0;
                    } else {
                        pos.0.x = wall.center().x - wall.w / 2.0 - size.0 / 2.0;
                    }
                } else {
                    if y_distance < 0.0 {
                        pos.0.y = wall.center().y + wall.h / 2.0 + size.0 / 2.0;
                    } else {
                        pos.0.y = wall.center().y - wall.h / 2.0 - size.0 / 2.0;
                    }
                };
            }
        }
    }
}

pub fn system_safe_zone_collision(area: &mut Area) {
    if area.safe_zones.is_empty() {
        return;
    }

    for (_, (dir, pos, size)) in area
        .world
        .query_mut::<Without<With<(&mut Direction, &Position, &Size), &BounceOffBounds>, &RenderReceiver>>()
    {
        for wall in &area.safe_zones {
            if wall.contains_circle(pos.0, size.0 / 2.0) {
                let distance = wall.center() - pos.0;

                let x_distance = distance.x / wall.w;
                let y_distance = distance.y / wall.h;

                if x_distance.abs() > y_distance.abs() {
                    dir.0.x *= -1.0;
                } else {
                    dir.0.y *= -1.0;
                }
            }
        }
    }

    for (_, (pos, size)) in area
        .world
        .query_mut::<Without<With<(&mut Position, &Size), &Bounded>, &RenderReceiver>>()
    {
        for wall in &area.safe_zones {
            if wall.contains_circle(pos.0, size.0 / 2.0) {
                let distance = wall.center() - pos.0;

                let x_distance = distance.x / wall.w;
                let y_distance = distance.y / wall.h;

                if x_distance.abs() > y_distance.abs() {
                    if x_distance < 0.0 {
                        pos.0.x = wall.center().x + wall.w / 2.0 + size.0 / 2.0;
                    } else {
                        pos.0.x = wall.center().x - wall.w / 2.0 - size.0 / 2.0;
                    }
                } else {
                    if y_distance < 0.0 {
                        pos.0.y = wall.center().y + wall.h / 2.0 + size.0 / 2.0;
                    } else {
                        pos.0.y = wall.center().y - wall.h / 2.0 - size.0 / 2.0;
                    }
                };
            }
        }
    }
}

pub fn system_enemy_collision(area: &mut Area) {
    let mut to_down = Vec::new();

    for (entity, (hero_pos, hero_size)) in area
        .world
        .query::<Without<With<(&Position, &Size), &Hero>, &Downed>>()
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
                to_down.push(entity);
            }
        }
    }

    for entity in to_down {
        let _ = area.world.insert_one(entity, Downed);

        let _ = area
            .leaderboard_tx
            .send(LeaderboardUpdate::set_downed(
                entity,
                area.full_id.clone(),
                true,
            ));
    }
}

pub fn system_hero_collision(area: &mut Area) {
    let mut to_revive = Vec::new();

    for (_, (pos_1, size_1)) in area
        .world
        .query::<Without<With<(&Position, &Size), &Hero>, &Downed>>()
        .iter()
    {
        for (entity, (pos_2, size_2)) in area
            .world
            .query::<With<(&Position, &Size), (&Hero, &Downed)>>()
            .iter()
        {
            let distance_sq = (pos_1.0 - pos_2.0).magnitude_sq();
            let radius_sum = (size_1.0 + size_2.0) * 0.5;

            if distance_sq < radius_sum * radius_sum {
                to_revive.push(entity);
            }
        }
    }

    for entity in to_revive {
        let result = area.world.remove_one::<Downed>(entity);

        if result.is_ok() {
            let _ = area
                .leaderboard_tx
                .send(LeaderboardUpdate::set_downed(
                    entity,
                    area.full_id.clone(),
                    false,
                ));
        }
    }
}

pub fn system_render(area: &mut Area) {
    area.render_packet = Some(RenderPacket::new());
    let nodes = &mut area.render_packet.as_mut().unwrap().nodes;

    for (entity, (pos, size, color, named, hero, enemy, downed)) in area.world.query_mut::<(
        &Position,
        &Size,
        &Color,
        Option<&Named>,
        Option<&Hero>,
        Option<&Enemy>,
        Option<&Downed>,
    )>() {
        let name = named.map(|n| n.0.clone());
        let mut color = color.clone();

        if downed.is_some() {
            color.a = 127;
        }

        let node = RenderNode {
            x: pos.0.x,
            y: pos.0.y,
            radius: size.0 / 2.0,
            color,
            has_border: enemy.is_some(),
            is_hero: hero.is_some(),
            downed: downed.is_some(),
            entity,
            name,
        };
        nodes.push(node);
    }
}

pub fn system_send_render_packet(area: &mut Area) {
    if let Some(packet) = &area.render_packet {
        for (entity, (render, pos)) in area.world.query_mut::<(&RenderReceiver, &Position)>() {
            if let Some(max_datagram_size) = render.connection.max_datagram_size() {
                let datagrams = packet.to_datagrams(max_datagram_size as u32, pos.0, entity);

                for datagram in datagrams {
                    let _ = render.connection.send_datagram(datagram);
                }
            }
        }
    }
}

pub async fn system_portals(area: &mut Area) {
    for (entity, (pos, size)) in area
        .world
        .query_mut::<With<(&mut Position, &Size), &Hero>>()
    {
        for portal in &area.portals {
            if portal.rect.contains_circle(pos.0, size.0 / 2.0) {
                let req = TransferRequest::new(
                    entity,
                    area.full_id.clone(),
                    portal.target_id.clone(),
                    Some(portal.target_pos),
                );

                let _ = area.transfer_tx.send(req).await;
            }
        }
    }
}
