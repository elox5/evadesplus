use super::{
    area::Area,
    components::{Downed, Position, RenderReceiver},
    data::MapData,
    systems::*,
    templates::MapTemplate,
};
use crate::{
    networking::{
        chat::{ChatMessageType, ChatRequest},
        leaderboard::{LeaderboardState, LeaderboardUpdate},
    },
    physics::vec2::Vec2,
};
use anyhow::Result;
use arc_swap::{ArcSwap, Guard};
use hecs::Entity;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::Duration,
};
use tokio::{
    join,
    sync::{
        broadcast,
        mpsc::{self},
        Mutex,
    },
    time::{interval, Instant},
};
use wtransport::Connection;

pub struct Game {
    maps: Vec<MapTemplate>,

    areas: HashMap<String, Arc<Mutex<Area>>>,

    players: Vec<Arc<ArcSwap<Player>>>,

    start_area_id: String,

    transfer_tx: mpsc::Sender<TransferRequest>,
    transfer_queue: Vec<u64>,

    pub leaderboard_state: LeaderboardState,

    leaderboard_tx: broadcast::Sender<LeaderboardUpdate>,
    pub leaderboard_rx: broadcast::Receiver<LeaderboardUpdate>,

    pub chat_tx: broadcast::Sender<ChatRequest>,
    pub chat_rx: broadcast::Receiver<ChatRequest>,

    frame_duration: Duration,
}

impl Game {
    pub fn new(maps: Vec<MapData>, start_area_id: &str) -> Arc<Mutex<Self>> {
        let (transfer_tx, mut transfer_rx) = mpsc::channel::<TransferRequest>(8);
        let (leaderboard_tx, leaderboard_rx) = broadcast::channel(8);

        let (chat_tx, chat_rx) = broadcast::channel(8);

        let mut lb_rx_clone = leaderboard_rx.resubscribe();

        let framerate: f32 = dotenvy::var("SIMULATION_FRAMERATE")
            .expect(".env SIMULATION_FRAMERATE must be set")
            .parse()
            .expect("Invalid framerate");

        let frame_duration = Duration::from_secs_f32(1.0 / framerate);

        let game = Game {
            maps: maps.into_iter().map(|m| m.to_template()).collect(),
            areas: HashMap::new(),
            players: Vec::new(),
            start_area_id: start_area_id.to_owned(),
            transfer_tx: transfer_tx.clone(),
            transfer_queue: Vec::new(),
            leaderboard_state: LeaderboardState::new(),
            leaderboard_tx,
            leaderboard_rx,
            chat_tx,
            chat_rx,
            frame_duration,
        };

        let arc = Arc::new(Mutex::new(game));
        let transfer_arc = arc.clone();
        let lb_arc = arc.clone();

        tokio::spawn(async move {
            while let Some(req) = transfer_rx.recv().await {
                let mut game = transfer_arc.lock().await;

                let _ = game.transfer_hero(req).await;
            }
        });

        tokio::spawn(async move {
            while let Ok(update) = lb_rx_clone.recv().await {
                let mut game = lb_arc.lock().await;

                game.leaderboard_state.update(update);
            }
        });

        arc
    }

    pub fn try_create_area(&mut self, id: &str) -> Result<Arc<Mutex<Area>>> {
        let (map_id, area_id) = Self::split_id(id).ok_or(anyhow::anyhow!("Invalid id"))?;

        let map = self
            .try_get_map(map_id)
            .ok_or(anyhow::anyhow!("Map '{}' not found", map_id))?;

        let template = map.get_area(area_id).ok_or(anyhow::anyhow!(
            "Area '{}' not found in map '{}'",
            area_id,
            map_id
        ))?;

        let area = Area::from_template(
            template,
            self.transfer_tx.clone(),
            self.leaderboard_tx.clone(),
        );
        let area = Arc::new(Mutex::new(area));
        Self::start_update_loop(area.clone(), self.frame_duration);
        self.areas.insert(id.to_owned(), area.clone());

        println!(
            "Area {} opened. Loaded areas: {:?}",
            id,
            self.areas.keys().collect::<Vec<_>>()
        );
        Ok(area)
    }

    pub fn get_or_create_area(&mut self, id: &str) -> Result<Arc<Mutex<Area>>> {
        if let Some(area) = self.areas.get(id) {
            return Ok(area.clone());
        }

        self.try_create_area(id)
    }

    pub fn close_area(&mut self, id: &str) {
        self.areas.remove(id);

        println!(
            "Area {} closed. Loaded areas: {:?}",
            id,
            self.areas.keys().collect::<Vec<_>>()
        );
    }

    pub fn get_start_area(&mut self) -> Result<Arc<Mutex<Area>>> {
        let start_area_id = self.start_area_id.clone();
        self.get_or_create_area(&start_area_id)
    }

    async fn update_area(area: &mut Area, delta_time: f32) {
        area.time += delta_time;
        area.delta_time = delta_time;

        system_update_velocity(area);
        system_update_position(area);
        system_bounds_check(area);

        system_inner_wall_collision(area);
        system_safe_zone_collision(area);
        system_portals(area).await;

        system_hero_collision(area);
        system_enemy_collision(area);

        system_render(area);
        system_send_render_packet(area);
    }

    fn start_update_loop(area: Arc<Mutex<Area>>, frame_duration: Duration) {
        let area_clone = area.clone();

        let handle = tokio::spawn(async move {
            let mut last_time = Instant::now();

            let mut interval = interval(frame_duration);

            loop {
                {
                    let mut area = area_clone.lock().await;
                    Self::update_area(&mut area, last_time.elapsed().as_secs_f32()).await;
                }

                last_time = Instant::now();
                interval.tick().await;
            }
        });

        area.try_lock().unwrap().loop_handle = Some(handle.abort_handle());
    }

    pub async fn spawn_hero(
        &mut self,
        id: u64,
        name: &str,
        connection: Connection,
    ) -> Arc<ArcSwap<Player>> {
        let start_area_id = self.start_area_id.clone();

        let area_arc = self
            .get_or_create_area(&start_area_id)
            .unwrap_or_else(|_| panic!("Start area '{start_area_id}' not found"));
        let mut area = area_arc.lock().await;

        let entity = area.spawn_player(name, connection);

        let player = Player::new(id, entity, area_arc.clone(), name.to_owned());
        let player = Arc::new(ArcSwap::new(Arc::new(player)));

        self.players.push(player.clone());

        let _ = self.leaderboard_tx.send(LeaderboardUpdate::add(
            entity,
            area.full_id.clone(),
            name.to_owned(),
            false,
            area.order,
            area.area_name.clone(),
            area.map_name.clone(),
        ));

        println!("Spawning hero '{}' (entity {})", name, entity.id());

        self.send_server_announcement(format!("{} joined the game", name));

        player
    }

    pub async fn despawn_hero(&mut self, player: &Arc<ArcSwap<Player>>) {
        if let Some(player_index) = self.players.iter().position(|p| Arc::ptr_eq(p, player)) {
            let name = player.load().name.clone();

            let player = self.players.swap_remove(player_index);
            let player = player.load();

            let mut area = player.area.lock().await;
            let (_, should_close) = area.despawn_player(player.entity);

            let _ = self.leaderboard_tx.send(LeaderboardUpdate::remove(
                player.entity,
                area.full_id.clone(),
            ));

            println!("Despawning hero '{}'", name);

            self.send_server_announcement(format!("{} left the game", name));

            if should_close {
                self.close_area(&area.full_id);
            }
        }
    }

    pub async fn reset_hero(&mut self, player: &Arc<ArcSwap<Player>>) -> Result<()> {
        let start_area_id = self.start_area_id.clone();

        let entity = player.load().entity;
        let current_area_id = player.load().area.lock().await.full_id.clone();

        let req = TransferRequest::new(entity, current_area_id.clone(), start_area_id, None);

        let _ = self.leaderboard_tx.send(LeaderboardUpdate::set_downed(
            entity,
            current_area_id,
            false,
        ));

        self.transfer_hero(req).await?;

        let player = player.load();
        let mut area = player.area.lock().await;

        let _ = area.world.remove_one::<Downed>(player.entity);

        Ok(())
    }

    pub async fn transfer_hero(&mut self, req: TransferRequest) -> Result<()> {
        if !self.transfer_queue.contains(&req.hash) {
            self.transfer_queue.push(req.hash);
        }

        if req.target_area_id == req.current_area_id {
            return self.move_hero_across_area(req).await;
        }

        let target_area_arc = self.get_or_create_area(&req.target_area_id)?;

        let player_arcswap = self
            .players
            .iter_mut()
            .find(|p| p.load().entity == req.entity)
            .unwrap();
        let player = player_arcswap.load();

        let (mut area, mut target_area) = join!(player.area.lock(), target_area_arc.lock());

        let (taken_entity, should_close) = area.despawn_player(req.entity);
        let entity = taken_entity?;
        let entity = target_area.world.spawn(entity);

        let _ = self.leaderboard_tx.send(LeaderboardUpdate::transfer(
            entity,
            req.entity,
            area.full_id.clone(),
            target_area.order,
            target_area.full_id.clone(),
            target_area.area_name.clone(),
            target_area.map_name.clone(),
        ));

        let target_pos = req.target_pos.unwrap_or(target_area.spawn_pos);

        let new_player = Player::new(
            player.id,
            entity,
            target_area_arc.clone(),
            player.name.clone(),
        );
        player_arcswap.store(Arc::new(new_player));

        if should_close {
            self.close_area(&area.full_id);
        }

        drop(area);

        let (render, pos) = target_area
            .world
            .query_one_mut::<(&RenderReceiver, &mut Position)>(entity)
            .unwrap();

        pos.0 = target_pos;

        self.transfer_queue.swap_remove(
            self.transfer_queue
                .iter()
                .position(|&hash| hash == req.hash)
                .unwrap(),
        );

        let mut response_stream = render.connection.open_uni().await?.await?;
        response_stream
            .write_all(&target_area.definition_packet())
            .await?;
        response_stream.finish().await?;

        println!("Transfer finished");

        Ok(())
    }

    pub async fn move_hero_across_area(&mut self, req: TransferRequest) -> Result<()> {
        let area = self.get_or_create_area(&req.current_area_id)?;

        let mut area = area.lock().await;

        let area_spawn_pos = area.spawn_pos;

        let pos = area.world.query_one_mut::<&mut Position>(req.entity)?;

        pos.0 = req.target_pos.unwrap_or_else(|| area_spawn_pos);

        Ok(())
    }

    pub async fn update_player_input(&mut self, entity: Entity, input: Vec2) {
        if let Some(player) = self.get_player(entity) {
            let mut area = player.area.lock().await;
            area.update_player_input(entity, input);
        }
    }

    pub fn get_player(&self, entity: Entity) -> Option<Guard<Arc<Player>>> {
        self.players
            .iter()
            .map(|p| p.load())
            .find(|p| p.entity == entity)
    }

    pub fn get_player_by_id(&self, id: u64) -> Option<Guard<Arc<Player>>> {
        self.players.iter().map(|p| p.load()).find(|p| p.id == id)
    }

    pub fn get_player_by_name(&self, name: &str) -> Option<Guard<Arc<Player>>> {
        self.players
            .iter()
            .map(|p| p.load())
            .find(|p| p.name == name)
    }

    fn split_id(id: &str) -> Option<(&str, &str)> {
        let mut split = id.split(':');
        let map_id = split.next()?;
        let area_id = split.next()?;
        Some((map_id, area_id))
    }

    fn try_get_map(&self, map_id: &str) -> Option<&MapTemplate> {
        self.maps.iter().find(|m| m.id == map_id)
    }

    fn send_server_announcement(&self, message: String) {
        let chat_broadcast = ChatRequest::new(
            message,
            String::new(),
            ChatMessageType::ServerAnnouncement,
            None,
        );
        let _ = self.chat_tx.send(chat_broadcast);
    }
}

pub struct Player {
    pub id: u64,
    pub entity: Entity,
    pub area: Arc<Mutex<Area>>,
    pub name: String,
}

impl Player {
    pub fn new(id: u64, entity: Entity, area: Arc<Mutex<Area>>, name: String) -> Self {
        Self {
            id,
            entity,
            area,
            name,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransferRequest {
    pub entity: Entity,
    pub current_area_id: String,
    pub target_area_id: String,
    pub target_pos: Option<Vec2>,
    hash: u64,
}

impl TransferRequest {
    pub fn new(
        entity: Entity,
        current_area_id: String,
        target_area_id: String,
        target_pos: Option<Vec2>,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        entity.id().hash(&mut hasher);
        current_area_id.hash(&mut hasher);
        target_area_id.hash(&mut hasher);

        Self {
            entity,
            current_area_id,
            target_area_id,
            target_pos,
            hash: hasher.finish(),
        }
    }
}
