use super::{
    components::{
        BounceOffBounds, Bounded, Color, Direction, Enemy, Hero, PlayerId, Position,
        RenderReceiver, Size, Speed, Timer, Velocity,
    },
    game::{TimerSyncPacket, TransferRequest},
    portal::{Portal, PortalCreationContext, PortalData},
};
use crate::{
    networking::{leaderboard::LeaderboardUpdate, rendering::RenderPacket},
    physics::{rect::Rect, vec2::Vec2},
};
use anyhow::Result;
use hecs::{Entity, TakenEntity, World};
use serde::Deserialize;
use tokio::{
    sync::{broadcast, mpsc},
    task::AbortHandle,
};
use wtransport::Connection;

pub struct Area {
    pub key: AreaKey,
    pub alias: Option<String>,

    pub name: String,
    pub map_name: String,
    pub full_name: String,

    pub background_color: Color,
    pub text_color: Option<Color>,
    pub message: Option<AreaMessage>,

    pub vp: Option<u8>,
    pub route_name: Option<String>,

    pub world: World,

    pub bounds: Rect,
    pub spawn_pos: Vec2,

    pub inner_walls: Vec<Rect>,
    pub safe_zones: Vec<Rect>,
    pub portals: Vec<Portal>,

    pub flags: AreaFlags,

    pub frame_count: u32,
    pub time: f32,
    pub delta_time: f32,

    pub render_packet: Option<RenderPacket>,

    pub loop_handle: Option<AbortHandle>,

    pub transfer_tx: mpsc::Sender<TransferRequest>,
    pub leaderboard_tx: broadcast::Sender<LeaderboardUpdate>,
    pub timer_sync_tx: broadcast::Sender<TimerSyncPacket>,
}

impl Area {
    pub fn new(
        template: &AreaTemplate,
        transfer_tx: mpsc::Sender<TransferRequest>,
        leaderboard_tx: broadcast::Sender<LeaderboardUpdate>,
        timer_sync_tx: broadcast::Sender<TimerSyncPacket>,
    ) -> Self {
        let mut area = Self {
            key: template.key.clone(),
            alias: template.alias.clone(),

            name: template.name.clone(),
            map_name: template.map_name.clone(),
            full_name: template.full_name.clone(),
            background_color: template.background_color.clone(),
            text_color: template.text_color.clone(),
            message: template.message.clone(),

            vp: template.vp,
            route_name: template.route_name.clone(),

            bounds: Rect::new(0.0, 0.0, template.width, template.height),
            spawn_pos: template.spawn_pos,
            inner_walls: template.inner_walls.clone(),
            safe_zones: template.safe_zones.clone(),
            portals: template.portals.clone(),
            flags: template.flags.clone(),
            world: World::new(),

            frame_count: 0,
            time: 0.0,
            delta_time: 0.0,
            render_packet: None,
            loop_handle: None,
            transfer_tx,
            leaderboard_tx,
            timer_sync_tx,
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

    pub fn spawn_player(&mut self, id: u64, connection: Connection) -> Entity {
        self.world.spawn((
            Position(self.spawn_pos),
            Velocity(Vec2::ZERO),
            Speed(17.0),
            Direction(Vec2::ZERO),
            Size(1.0),
            Color::rgb(rand::random(), rand::random(), rand::random()),
            RenderReceiver { connection },
            PlayerId(id),
            Hero,
            Bounded,
            Timer(0.0),
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

        let flags = self.flags.boss as u8
            | (self.flags.victory as u8) << 1
            | (self.text_color.is_some() as u8) << 2;

        packet.push(flags);

        packet.extend_from_slice(&self.background_color.to_bytes());

        packet.push(self.name.len() as u8);
        packet.extend_from_slice(self.name.as_bytes());

        packet.push(self.key.map_id.len() as u8);
        packet.extend_from_slice(self.key.map_id.as_bytes());

        if let Some(color) = &self.text_color {
            packet.extend_from_slice(&color.to_bytes());
        }

        if let Some(message) = &self.message {
            packet.extend_from_slice(&message.to_bytes());
        } else {
            packet.push(0);
        }

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

pub struct AreaTemplate {
    pub key: AreaKey,
    pub alias: Option<String>,

    pub name: String,
    pub map_name: String,
    pub full_name: String,
    pub background_color: Color,
    pub text_color: Option<Color>,
    pub message: Option<AreaMessage>,

    pub vp: Option<u8>,
    pub route_name: Option<String>,

    pub width: f32,
    pub height: f32,

    pub spawn_pos: Vec2,

    pub inner_walls: Vec<Rect>,
    pub safe_zones: Vec<Rect>,
    pub portals: Vec<Portal>,

    pub enemy_groups: Vec<EnemyGroup>,

    pub flags: AreaFlags,
}

impl AreaTemplate {
    pub fn new(data: AreaData, order: u16, ctx: &AreaCreationContext) -> Self {
        let key = AreaKey::new(ctx.map_id.clone(), order as u16);

        let name = data.name.unwrap_or_else(|| format!("Area {}", order + 1));

        let background_color = match data.background_color {
            Some(color) => color,
            None => match data.flags.as_ref().map(|f| f.victory).flatten() {
                Some(true) => "ffff50".to_owned(),
                _ => ctx.background_color.clone(),
            },
        }
        .into();

        let text_color = match data.text_color {
            Some(color) => Some(Color::from_hex(&color)),
            None => match data.flags.as_ref().map(|f| f.victory).flatten() {
                Some(true) => Some(Color::from_hex("#ffff00")),
                _ => None,
            },
        };

        let width = data.width.unwrap_or(100.0);
        let height = data.height.unwrap_or(15.0);

        let portal_ctx = PortalCreationContext {
            map_id: ctx.map_id.clone(),
            area_order: order,
        };

        let portals = match data.portals {
            Some(portals) => portals
                .into_iter()
                .map(|data| Portal::new(data, &portal_ctx))
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };

        let enemy_groups = data.enemy_groups.unwrap_or_default();
        let enemy_groups = enemy_groups
            .into_iter()
            .map(|group| EnemyGroup {
                color: group.color.into(),
                count: group.count,
                speed: group.speed,
                size: group.size,
            })
            .collect::<Vec<_>>();

        let message = data
            .message
            .map(|message| AreaMessage::new(message, data.message_config, data.vp));

        AreaTemplate {
            key,
            alias: data.alias,
            name: name.clone(),
            map_name: ctx.map_name.clone(),
            full_name: format!("{} - {}", ctx.map_name, name),
            background_color,
            text_color,
            message,
            vp: data.vp,
            route_name: data
                .route_name
                .map(|route| format!("{} - {}", ctx.map_name, route)),
            width,
            height,
            spawn_pos: data
                .spawn_pos
                .unwrap_or_else(|| Vec2::new(5.0, height / 2.0)),
            portals,
            inner_walls: data.inner_walls.unwrap_or_default(),
            safe_zones: data.safe_zones.unwrap_or_default(),
            enemy_groups,
            flags: AreaFlags::new(data.flags),
        }
    }
}

pub struct AreaCreationContext {
    pub map_id: String,
    pub map_name: String,
    pub background_color: String,
}

#[derive(Deserialize)]
pub struct AreaData {
    pub alias: Option<String>,
    pub name: Option<String>,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub message: Option<String>,
    pub message_config: Option<MessageConfigData>,

    pub vp: Option<u8>,
    pub route_name: Option<String>,

    pub width: Option<f32>,
    pub height: Option<f32>,

    pub spawn_pos: Option<Vec2>,

    pub inner_walls: Option<Vec<Rect>>,
    pub safe_zones: Option<Vec<Rect>>,
    pub portals: Option<Vec<PortalData>>,

    pub enemy_groups: Option<Vec<EnemyGroupData>>,

    pub flags: Option<AreaFlagsData>,
}

#[derive(Clone)]
pub struct AreaFlags {
    pub boss: bool,
    pub victory: bool,
    pub final_victory: bool,
}

impl AreaFlags {
    pub fn new(data: Option<AreaFlagsData>) -> Self {
        match data {
            Some(data) => Self {
                boss: data.boss.unwrap_or(false),
                victory: data.victory.unwrap_or(false),
                final_victory: data.final_victory.unwrap_or(false),
            },
            None => Self {
                boss: false,
                victory: false,
                final_victory: false,
            },
        }
    }
}

#[derive(Deserialize)]
pub struct AreaFlagsData {
    pub boss: Option<bool>,
    pub victory: Option<bool>,
    pub final_victory: Option<bool>,
}

#[derive(Clone)]
pub struct EnemyGroup {
    pub color: Color,
    pub count: u32,
    pub speed: f32,
    pub size: f32,
}

impl EnemyGroup {
    pub fn new(color: Color, count: u32, speed: f32, size: f32) -> Self {
        Self {
            color,
            count,
            speed,
            size,
        }
    }
}

#[derive(Deserialize)]
pub struct EnemyGroupData {
    pub color: String,
    pub count: u32,
    pub speed: f32,
    pub size: f32,
}

#[derive(Clone)]
pub struct AreaMessage {
    pub message: String,
    pub color: Color,
}

impl AreaMessage {
    pub fn new(message: String, data: Option<MessageConfigData>, vp: Option<u8>) -> Self {
        let color = data
            .map(|data| data.color)
            .flatten()
            .unwrap_or("#7fff7f".to_owned());

        let mut message = message;

        if let Some(vp) = vp {
            message.push_str(&format!("\n{vp} VP awarded!"));
        }

        Self {
            message,
            color: color.into(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.message.len() as u8);
        bytes.extend_from_slice(self.message.as_bytes());

        bytes.extend_from_slice(&self.color.to_bytes());

        bytes
    }
}

#[derive(Deserialize)]
pub struct MessageConfigData {
    pub color: Option<String>,
}
