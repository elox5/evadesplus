use super::{
    components::{
        BounceOffBounds, Bounded, Color, Direction, Enemy, Hero, Named, PlayerId, Position,
        RenderReceiver, Size, Speed, Velocity,
    },
    game::TransferRequest,
    templates::{AreaTemplate, EnemyGroup},
};
use crate::{
    networking::{leaderboard::LeaderboardUpdate, rendering::RenderPacket},
    physics::{rect::Rect, vec2::Vec2},
};
use anyhow::Result;
use hecs::{Entity, TakenEntity, World};
use tokio::{
    sync::{broadcast, mpsc},
    task::AbortHandle,
};
use wtransport::Connection;

pub struct Area {
    pub key: AreaKey,
    pub alias: Option<String>,

    pub name: String,
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

    pub transfer_tx: mpsc::Sender<TransferRequest>,
    pub leaderboard_tx: broadcast::Sender<LeaderboardUpdate>,
}

impl Area {
    pub fn from_template(
        template: &AreaTemplate,
        transfer_tx: mpsc::Sender<TransferRequest>,
        leaderboard_tx: broadcast::Sender<LeaderboardUpdate>,
    ) -> Self {
        let mut area = Self {
            key: template.key.clone(),
            alias: template.alias.clone(),

            name: template.name.clone(),

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
            leaderboard_tx,
        };

        for group in &template.enemy_groups {
            area.spawn_enemy_group(group);
        }

        area
    }

    fn close(&mut self) {
        if let Some(handle) = self.loop_handle.take() {
            handle.abort();
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

    pub fn spawn_player(&mut self, id: u64, name: &str, connection: Connection) -> Entity {
        self.world.spawn((
            Position(self.spawn_pos),
            Velocity(Vec2::ZERO),
            Speed(17.0),
            Direction(Vec2::ZERO),
            Size(1.0),
            Color::rgb(rand::random(), rand::random(), rand::random()),
            Named(name.to_owned()),
            RenderReceiver { connection },
            PlayerId(id),
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

        packet.push(self.name.len() as u8);
        packet.extend_from_slice(self.name.as_bytes());

        packet.push(self.key.map_id.len() as u8);
        packet.extend_from_slice(self.key.map_id.as_bytes());

        packet
    }
}
#[derive(Eq, Clone)]
pub struct AreaKey {
    map_id: String,
    order: u16,
}

impl AreaKey {
    pub fn new(map_id: String, order: u16) -> Self {
        Self { map_id, order }
    }

    pub fn from_map_order_string(string: &str) -> Result<Self> {
        let (map_id, order) = string
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid area key format"))?;

        let key = Self {
            map_id: map_id.to_owned(),
            order: order.parse()?,
        };

        Ok(key)
    }

    pub fn map_id(&self) -> &str {
        &self.map_id
    }

    pub fn order(&self) -> u16 {
        self.order
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.map_id, self.order)
    }
}

impl std::fmt::Debug for AreaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for AreaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::hash::Hash for AreaKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.map_id.hash(state);
        self.order.hash(state);
    }
}

impl PartialEq for AreaKey {
    fn eq(&self, other: &Self) -> bool {
        self.map_id == other.map_id && self.order == other.order
    }
}

#[derive(Clone)]
pub struct Portal {
    pub rect: Rect,
    pub color: Color,
    pub target_key: AreaKey,
    pub target_pos: Vec2,
}
