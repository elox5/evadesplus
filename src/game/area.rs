use super::{
    components::{
        BounceOffBounds, Bounded, Color, Direction, Enemy, Hero, Player, Position, Size, Speed,
        Velocity,
    },
    templates::{AreaTemplate, EnemyGroup},
};
use crate::{
    networking::rendering::RenderPacket,
    physics::{rect::Rect, vec2::Vec2},
};
use hecs::{Entity, World};
use tokio::{sync::mpsc, task::AbortHandle};
use wtransport::Connection;

pub struct Area {
    pub area_id: String,
    pub full_id: String,
    pub name: String,
    pub background_color: Color,

    pub world: World,

    pub bounds: Rect,
    pub inner_walls: Vec<Rect>,
    pub safe_zones: Vec<Rect>,
    pub portals: Vec<Portal>,

    pub time: f32,
    pub delta_time: f32,

    pub render_packet: Option<RenderPacket>,

    pub loop_handle: Option<AbortHandle>,

    pub transfer_tx: mpsc::Sender<(Entity, String)>,
}

impl Area {
    pub fn from_template(
        template: &AreaTemplate,
        transfer_tx: mpsc::Sender<(Entity, String)>,
    ) -> Self {
        let mut area = Self {
            area_id: template.area_id.clone(),
            full_id: template.full_id.clone(),
            name: template.name.clone(),
            background_color: template.background_color.clone(),
            bounds: Rect::new(0.0, 0.0, template.width, template.height),
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
            area.spawn_enemy_group(&group);
        }

        area
    }

    pub fn close(&mut self) {
        if let Some(handle) = self.loop_handle.take() {
            handle.abort();
        }
    }

    pub fn spawn_enemy_group(&mut self, group: &EnemyGroup) {
        let enemies = (0..group.count).map(|_| {
            let pos = Position(self.bounds.random_inside());
            let vel = Velocity(Vec2::ZERO);
            let dir = Direction(Vec2::random_unit());
            let speed = Speed(group.speed);
            let size = Size(group.size);
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

    pub fn despawn_player(&mut self, entity: Entity) {
        let _ = self.world.despawn(entity);
        println!("Despawning player");
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

        packet.extend_from_slice(&self.name.len().to_le_bytes()[..4]);
        packet.extend_from_slice(self.name.as_bytes());

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
