use super::{
    components::{
        BounceOffBounds, Bounded, Color, Direction, Enemy, Hero, Named, Position, RenderReceiver,
        Size, Speed, Velocity,
    },
    templates::{AreaTemplate, EnemyGroup},
};
use crate::{
    networking::rendering::RenderPacket,
    physics::{rect::Rect, vec2::Vec2},
};
use hecs::{Entity, TakenEntity, World};
use tokio::{sync::mpsc, task::AbortHandle};
use wtransport::Connection;

pub struct Area {
    pub order: u16,

    pub area_name: String,
    pub map_name: String,
    pub full_name: String,

    pub area_id: String,
    pub map_id: String,
    pub full_id: String,

    pub background_color: Color,

    pub world: World,

    pub bounds: Rect,
    pub spawn_pos: Vec2,

    pub inner_walls: Vec<Rect>,
    pub safe_zones: Vec<Rect>,
    pub portals: Vec<Portal>,

    pub time: f32,
    pub delta_time: f32,

    pub render_packet: Option<RenderPacket>,

    pub loop_handle: Option<AbortHandle>,

    pub transfer_tx: mpsc::Sender<(Entity, String, Vec2)>,
}

impl Area {
    pub fn from_template(
        template: &AreaTemplate,
        transfer_tx: mpsc::Sender<(Entity, String, Vec2)>,
    ) -> Self {
        let mut area = Self {
            order: template.order,

            area_name: template.area_name.clone(),
            map_name: template.map_name.clone(),
            full_name: format!("{} - {}", template.map_name, template.area_name),

            area_id: template.area_id.clone(),
            map_id: template.map_id.clone(),
            full_id: format!("{}:{}", template.map_id, template.area_id),

            background_color: template.background_color.clone(),
            bounds: Rect::new(0.0, 0.0, template.width, template.height),
            spawn_pos: template.spawn_pos,
            inner_walls: template.inner_walls.clone(),
            safe_zones: template.safe_zones.clone(),
            portals: template.portals.clone(),
            world: World::new(),
            time: 0.0,
            delta_time: 0.0,
            render_packet: None,
            loop_handle: None,
            transfer_tx,
        };

        for group in &template.enemy_groups {
            area.spawn_enemy_group(group);
        }

        area
    }

    fn close(&mut self) {
        if let Some(handle) = self.loop_handle.take() {
            handle.abort();
            println!("Area {} closed", self.full_id);
        }
    }

    pub fn spawn_enemy_group(&mut self, group: &EnemyGroup) {
        let enemies = (0..group.count).map(|_| {
            let size = Size(group.size);

            let mut pos = Position(self.bounds.random_inside());

            while self
                .safe_zones
                .iter()
                .chain(self.inner_walls.iter())
                .any(|zone| zone.contains_circle(pos.0, size.0))
            {
                pos = Position(self.bounds.random_inside());
            }

            let vel = Velocity(Vec2::ZERO);
            let dir = Direction(Vec2::random_unit());
            let speed = Speed(group.speed);
            let color = group.color.clone();

            (
                Enemy,
                pos,
                vel,
                dir,
                speed,
                size,
                color,
                Bounded,
                BounceOffBounds,
            )
        });

        self.world.spawn_batch(enemies);
    }

    pub fn spawn_player(&mut self, name: &str, connection: Connection) -> Entity {
        self.world.spawn((
            Position(self.spawn_pos),
            Velocity(Vec2::ZERO),
            Speed(17.0),
            Direction(Vec2::ZERO),
            Size(1.0),
            Color::rgb(rand::random(), rand::random(), rand::random()),
            Named(name.to_owned()),
            RenderReceiver { connection },
            Hero,
            Bounded,
        ))
    }

    pub fn despawn_player(
        &mut self,
        entity: Entity,
    ) -> (Result<TakenEntity<'_>, hecs::NoSuchEntity>, bool) {
        let hero_count = self.world.query_mut::<&Hero>().into_iter().count();
        let should_close = hero_count == 1;

        if should_close {
            self.close();
        }

        (self.world.take(entity), should_close)
    }

    pub fn update_player_input(&mut self, entity: Entity, input: Vec2) {
        let dir = self.world.query_one_mut::<&mut Direction>(entity);
        if let Ok(dir) = dir {
            dir.0 = input;
        }
    }

    pub fn definition_packet(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(b"ADEF"); // area definition
        packet.extend_from_slice(&self.bounds.w.to_le_bytes());
        packet.extend_from_slice(&self.bounds.h.to_le_bytes());
        packet.extend_from_slice(&self.background_color.to_bytes());

        packet.extend_from_slice(&(self.inner_walls.len() as u16).to_le_bytes());
        packet.extend_from_slice(&(self.safe_zones.len() as u16).to_le_bytes());
        packet.extend_from_slice(&(self.portals.len() as u16).to_le_bytes());

        for wall in &self.inner_walls {
            packet.extend_from_slice(&wall.to_bytes());
        }

        for zone in &self.safe_zones {
            packet.extend_from_slice(&zone.to_bytes());
        }

        for portal in &self.portals {
            packet.extend_from_slice(&portal.rect.to_bytes());
            packet.extend_from_slice(&portal.color.to_bytes());
        }

        packet.extend_from_slice(&self.full_name.len().to_le_bytes()[..4]);
        packet.extend_from_slice(self.full_name.as_bytes());

        packet
    }
}

#[derive(Clone)]
pub struct Portal {
    pub rect: Rect,
    pub color: Color,
    pub target_id: String,
    pub target_pos: Vec2,
}
