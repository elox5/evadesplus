use super::{
    components::{Bounded, Color, Direction, Hero, Player, Position, Size, Speed, Velocity},
    systems::*,
};
use crate::{
    networking::rendering::RenderPacket,
    physics::{rect::Rect, vec2::Vec2},
};
use hecs::{Entity, World};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{interval, Instant},
};
use wtransport::Connection;

pub struct Area {
    pub name: String,
    pub id: String,

    pub world: World,

    pub bounds: Rect,

    pub time: f32,
    pub delta_time: f32,

    pub render_packet: Option<RenderPacket>,
}

impl Area {
    pub fn new(id: String, name: String, width: f32, height: f32) -> Self {
        Self {
            name,
            id,
            world: World::new(),
            bounds: Rect::new(0.0, 0.0, width, height),
            time: 0.0,
            delta_time: 0.0,
            render_packet: None,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        self.delta_time = delta_time;

        system_update_velocity(self);
        system_update_position(self);
        system_bounds_check(self);

        system_hero_collision(self);
        system_enemy_collision(self);

        system_render(self);
        system_send_render_packet(self);
    }

    pub fn start_update_loop(area: Arc<Mutex<Area>>) {
        tokio::spawn(async move {
            let mut last_time = Instant::now();

            let mut interval = interval(Duration::from_millis(16));

            loop {
                {
                    let mut area = area.lock().await;
                    area.update(last_time.elapsed().as_secs_f32());
                }

                last_time = Instant::now();
                interval.tick().await;
            }
        });
    }

    pub fn spawn_hero(&mut self, name: &str, connection: Connection) -> Entity {
        let player = Player {
            connection,
            name: name.to_owned(),
        };

        let pos = Position(self.bounds.center());
        let vel = Velocity(Vec2::ZERO);
        let speed = Speed(17.0);
        let dir = Direction(Vec2::ZERO);

        let size = Size(1.0);
        let color = Color::rgb(rand::random(), rand::random(), rand::random());

        self.world
            .spawn((player, Hero, pos, vel, speed, dir, size, color, Bounded))
    }

    pub fn update_hero_dir(&mut self, entity: Entity, new_dir: Vec2) {
        let dir = self.world.query_one_mut::<&mut Direction>(entity).unwrap();
        dir.0 = new_dir;
    }
}
