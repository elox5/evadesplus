use super::{area::Area, components::*};
use crate::{
    game::{
        components::{Direction, Position, Speed, Velocity},
        game::PlayerStatusMessage,
        player::PlayerId,
        transfer_request::{
            TransferRequest, TransferRequestTargetPos, TransferRequestTargetPosX,
            TransferRequestTargetPosY, TransferTarget,
        },
    },
    networking::rendering::{AreaRenderPacket, RenderNode},
    physics::vec2::Vec2,
};
use hecs::{With, Without};

pub fn system_increment_timer(area: &mut Area) {
    for (_, timer) in area.world.query_mut::<&mut Timer>() {
        timer.0 += area.delta_time;
    }
}

pub fn system_evaluate_target_position(area: &mut Area) {
    for (_, (pos, target_pos, vel)) in area
        .world
        .query_mut::<Without<(&Position, &mut TargetPosition, &Velocity), &Downed>>()
    {
        target_pos.0 = pos.0 + vel.0 * area.delta_time;
    }
}

pub fn system_commit_position(area: &mut Area) {
    for (_, (pos, target_pos)) in area.world.query_mut::<(&mut Position, &TargetPosition)>() {
        pos.0 = target_pos.0;
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
        .query_mut::<With<(&mut Direction, &TargetPosition, &Size), &BounceOffBounds>>()
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
        .query_mut::<With<(&mut TargetPosition, &Size), &Bounded>>()
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

    for (_, (pos, target_pos, size, dir, bounce, hero)) in area.world.query_mut::<With<
        (
            &Position,
            &mut TargetPosition,
            &Size,
            &mut Direction,
            Option<&BounceOffBounds>,
            Option<&Hero>,
        ),
        &Bounded,
    >>() {
        let radius = size.0 / 2.0;

        let mut current_sub_pos = pos.0;
        let total_vel = target_pos.0 - pos.0;
        let speed = total_vel.magnitude();

        if speed < 0.0001 {
            continue;
        }

        let max_step_distance = (radius * 0.5).max(0.05);
        let substeps = ((speed / max_step_distance).ceil() as usize).clamp(1, 32);

        let sub_vel = total_vel / substeps as f32;

        for _step in 0..substeps {
            current_sub_pos += sub_vel;

            for wall in &area.inner_walls {
                let closest_x = current_sub_pos.x.clamp(wall.min().x, wall.max().x);
                let closest_y = current_sub_pos.y.clamp(wall.min().y, wall.max().y);
                let closest_point = Vec2::new(closest_x, closest_y);

                let to_circle = current_sub_pos - closest_point;
                let distance = to_circle.magnitude();

                if distance < radius {
                    let normal = if distance > 0.0001 {
                        to_circle.normalized()
                    } else {
                        Vec2::UP
                    };
                    let penetration = radius - distance;

                    current_sub_pos += normal * penetration;

                    let mut local_dir = dir.0;
                    let dot = dir.0.dot(&normal);
                    if dot < 0.0 {
                        if bounce.is_some() {
                            local_dir = local_dir - (normal * 2.0 * dot);
                        } else {
                            local_dir = local_dir - (normal * dot);
                        }
                        local_dir = local_dir.normalized();
                    }

                    if hero.is_none() {
                        dir.0 = local_dir;
                    }
                }
            }
        }

        target_pos.0 = current_sub_pos;
    }
}

pub fn system_safe_zone_collision(area: &mut Area) {
    if area.safe_zones.is_empty() {
        return;
    }

    for (_, (pos, target_pos, size, dir)) in
        area.world.query_mut::<With<
            (&Position, &mut TargetPosition, &Size, &mut Direction),
            (&Bounded, &SafeZoneBounded),
        >>()
    {
        let radius = size.0 / 2.0;

        let mut current_sub_pos = pos.0;
        let total_vel = target_pos.0 - pos.0;
        let speed = total_vel.magnitude();

        if speed < 0.0001 {
            continue;
        }

        let max_step_distance = (radius * 0.5).max(0.05);
        let substeps = ((speed / max_step_distance).ceil() as usize).clamp(1, 32);

        let sub_vel = total_vel / substeps as f32;

        for _step in 0..substeps {
            current_sub_pos += sub_vel;

            for wall in &area.safe_zones {
                let closest_x = current_sub_pos.x.clamp(wall.min().x, wall.max().x);
                let closest_y = current_sub_pos.y.clamp(wall.min().y, wall.max().y);
                let closest_point = Vec2::new(closest_x, closest_y);

                let to_circle = current_sub_pos - closest_point;
                let distance = to_circle.magnitude();

                if distance < radius {
                    let normal = if distance > 0.0001 {
                        to_circle.normalized()
                    } else {
                        Vec2::UP
                    };
                    let penetration = radius - distance;

                    current_sub_pos += normal * penetration;

                    let dot = dir.0.dot(&normal);
                    if dot < 0.0 {
                        dir.0 = dir.0 - (normal * 2.0 * dot);
                        dir.0 = dir.0.normalized();
                    }
                }
            }
        }

        target_pos.0 = current_sub_pos;
    }
}

pub async fn system_enemy_collision(area: &mut Area) {
    let mut to_down = Vec::new();

    for (entity, (hero_pos, hero_size)) in area
        .world
        .query::<Without<Without<With<(&Position, &Size), &Hero>, &CrossingPortal>, &Downed>>()
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
            .status_tx
            .send(PlayerStatusMessage {
                player_id: PlayerId {
                    entity,
                    area: area.key.clone(),
                },
                alive: false,
            })
            .await;
    }
}

pub async fn system_hero_collision(area: &mut Area) {
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
                .status_tx
                .send(PlayerStatusMessage {
                    player_id: PlayerId {
                        entity,
                        area: area.key.clone(),
                    },
                    alive: true,
                })
                .await;
        }
    }
}

pub fn system_render(area: &mut Area) {
    area.render_packet = Some(AreaRenderPacket::new());
    let nodes = &mut area.render_packet.as_mut().unwrap().nodes;

    for (entity, (pos, size, color, hero, enemy, downed)) in area.world.query_mut::<(
        &Position,
        &Size,
        &Color,
        Option<&Hero>,
        Option<&Enemy>,
        Option<&Downed>,
    )>() {
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
            entity: Some(entity),
            user_id: None,
        };
        nodes.push(node);
    }
}

pub fn system_sync_timers(area: &mut Area) {
    // for (_, (timer)) in area.world.query_mut::<(&mut Timer)>() {
    //     let _ = area.timer_sync_tx.send(TimerSyncPacket {
    //         player_id: player_id.clone(),
    //         time: timer.0,
    //     });
    // }
}

pub async fn system_portals(area: &mut Area) {
    let mut to_cross = Vec::new();

    for (entity, (pos, size)) in area
        .world
        .query_mut::<With<(&mut Position, &Size), &Hero>>()
    {
        for portal in &area.portals {
            if portal.rect.contains_circle(pos.0, size.0 / 2.0) {
                let area_key = portal.target.get_area_key();

                if let Ok(target_area_key) = area_key {
                    let req = TransferRequest {
                        player: PlayerId {
                            entity,
                            area: area.key.clone(),
                        },
                        target: TransferTarget::Area(target_area_key),
                        target_pos: Some(TransferRequestTargetPos {
                            x: TransferRequestTargetPosX::new(portal.target_x.clone(), pos.0.x),
                            y: TransferRequestTargetPosY::new(portal.target_y.clone(), pos.0.y),
                        }),
                    };

                    to_cross.push(entity);

                    let _ = area.transfer_tx.send(req).await;
                }
            }
        }
    }

    for entity in to_cross {
        let _ = area.world.insert_one(entity, CrossingPortal);
    }
}
