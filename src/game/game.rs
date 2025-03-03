use super::{
    area::{Area, AreaKey},
    components::{CrossingPortal, Downed, Position, RenderReceiver},
    map_table::try_get_map,
    systems::*,
};
use crate::{
    env::get_env_or_default,
    networking::{
        chat::{ChatMessageType, ChatRequest},
        leaderboard::{LeaderboardState, LeaderboardUpdate},
    },
    physics::vec2::Vec2,
};
use anyhow::{anyhow, Result};
use arc_swap::{ArcSwap, Guard};
use hecs::Entity;
use std::{collections::HashMap, sync::Arc, time::Duration};
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
    areas: HashMap<AreaKey, Arc<Mutex<Area>>>,

    players: HashMap<u64, ArcSwap<Player>>,

    spawn_area_key: AreaKey,

    transfer_tx: mpsc::Sender<TransferRequest>,
    transfer_queue: Vec<u64>,

    pub leaderboard_state: LeaderboardState,

    leaderboard_tx: broadcast::Sender<LeaderboardUpdate>,
    pub leaderboard_rx: broadcast::Receiver<LeaderboardUpdate>,

    pub chat_tx: broadcast::Sender<ChatRequest>,

    frame_duration: Duration,
}

impl Game {
    pub fn new(start_map_id: String) -> Arc<Mutex<Self>> {
        let (transfer_tx, mut transfer_rx) = mpsc::channel::<TransferRequest>(8);
        let (leaderboard_tx, leaderboard_rx) = broadcast::channel(8);

        let (chat_tx, _) = broadcast::channel(8);

        let mut lb_rx_clone = leaderboard_rx.resubscribe();

        let framerate: f32 = get_env_or_default("SIMULATION_FRAMERATE", "60")
            .parse()
            .expect("Invalid framerate");

        let frame_duration = Duration::from_secs_f32(1.0 / framerate);

        let spawn_area_key = try_get_map(&start_map_id)
            .unwrap_or_else(|| panic!("Could not find start map"))
            .get_start_area()
            .key
            .clone();

        let game = Game {
            areas: HashMap::new(),
            players: HashMap::new(),
            spawn_area_key,
            transfer_tx: transfer_tx.clone(),
            transfer_queue: Vec::new(),
            leaderboard_state: LeaderboardState::new(),
            leaderboard_tx,
            leaderboard_rx,
            chat_tx,
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

    fn try_create_area(&mut self, key: &AreaKey) -> Result<Arc<Mutex<Area>>> {
        let map_id = key.map_id();

        let map =
            try_get_map(map_id).ok_or_else(|| anyhow::anyhow!("Map '{}' not found", map_id))?;

        let template = map
            .try_get_area(key.order() as usize)
            .ok_or_else(|| anyhow::anyhow!("Area '{}' not found", key))?;

        println!(
            "Area {} opened. Loaded areas: {:?}",
            key,
            self.areas.keys().collect::<Vec<_>>()
        );

        let area = Area::new(
            template,
            self.transfer_tx.clone(),
            self.leaderboard_tx.clone(),
        );
        let area = Arc::new(Mutex::new(area));
        Self::start_update_loop(area.clone(), self.frame_duration);
        self.areas.insert(key.clone(), area.clone());

        Ok(area)
    }

    pub fn get_or_create_area(&mut self, key: &AreaKey) -> Result<Arc<Mutex<Area>>> {
        if let Some(area) = self.areas.get(&key) {
            return Ok(area.clone());
        }

        self.try_create_area(key)
    }

    fn close_area(&mut self, key: &AreaKey) {
        self.areas.remove(key);

        println!(
            "Area {} closed. Loaded areas: {:?}",
            key,
            self.areas.keys().collect::<Vec<_>>()
        );
    }

    fn get_spawn_area(&mut self) -> Arc<Mutex<Area>> {
        let spawn_area_key = self.spawn_area_key.clone();
        self.get_or_create_area(&spawn_area_key)
            .unwrap_or_else(|_| panic!("Start area '{spawn_area_key}' not found"))
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

    pub async fn spawn_hero(&mut self, id: u64, name: &str, connection: Connection) {
        let area = self.get_spawn_area();
        let mut area = area.lock().await;

        let entity = area.spawn_player(id, connection);

        let player = Player::new(id, name.to_owned(), entity, area.key.clone());
        self.players.insert(id, ArcSwap::new(Arc::new(player)));

        let _ = self.leaderboard_tx.send(LeaderboardUpdate::add(
            id,
            name.to_owned(),
            false,
            area.key.order(),
            area.name.clone(),
            area.text_color.clone().map(|c| c.to_hex()),
            area.key.map_id().to_owned(),
        ));

        println!("Spawning hero '{}' (entity {})", name, entity.id());

        self.send_server_announcement(format!("{} joined the game", name));
    }

    pub async fn despawn_hero(&mut self, player_id: u64) -> Result<()> {
        let player = self.get_player(player_id)?;

        let area_arc = self.get_or_create_area(&player.area_key)?;
        let mut area = area_arc.lock().await;

        let (_, should_close) = area.despawn_player(player.entity);

        let _ = self
            .leaderboard_tx
            .send(LeaderboardUpdate::remove(player_id));

        println!("Despawning hero '{}'", player.name);

        self.send_server_announcement(format!("{} left the game", player.name));

        if should_close {
            self.close_area(&area.key);
        }

        self.players.remove(&player_id);

        Ok(())
    }

    pub async fn reset_hero(&mut self, player_id: u64) -> Result<()> {
        let req = TransferRequest {
            player_id,
            target: TransferTarget::Spawn,
            target_pos: None,
        };

        self.transfer_hero(req).await?;

        let player = self.get_player(player_id)?;

        let _ = self
            .leaderboard_tx
            .send(LeaderboardUpdate::set_downed(player_id, false));

        let area = self.get_or_create_area(&player.area_key)?;
        let mut area = area.lock().await;

        let _ = area.world.remove_one::<Downed>(player.entity);

        Ok(())
    }

    pub async fn transfer_hero(&mut self, req: TransferRequest) -> Result<()> {
        if !self.transfer_queue.contains(&req.player_id) {
            self.transfer_queue.push(req.player_id);
        }

        let player = self.get_player(req.player_id)?;

        let target_key = match req.target {
            TransferTarget::Spawn => self.spawn_area_key.clone(),
            TransferTarget::MapStart(ref map_id) => try_get_map(&map_id)
                .ok_or_else(|| anyhow::anyhow!("Map '{}' not found", map_id))?
                .get_start_area()
                .key
                .clone(),
            TransferTarget::Area(ref key) => key.clone(),
        };

        if target_key == player.area_key {
            return self.move_hero_across_area(req).await;
        }

        let target_area_arc = self.get_or_create_area(&target_key)?;

        let player_area = self.get_or_create_area(&player.area_key)?;

        let (mut player_area, mut target_area) = join!(player_area.lock(), target_area_arc.lock());

        let (taken_entity, should_close) = player_area.despawn_player(player.entity);
        let entity = taken_entity?;
        let entity = target_area.world.spawn(entity);

        let _ = target_area.world.remove_one::<CrossingPortal>(entity);

        let _ = self.leaderboard_tx.send(LeaderboardUpdate::transfer(
            req.player_id,
            target_area.key.order(),
            target_area.name.clone(),
            target_area.text_color.clone().map(|c| c.to_hex()),
            target_area.key.map_id().to_owned(),
        ));

        let target_pos = req.target_pos.unwrap_or(target_area.spawn_pos);

        let mut new_player = Player {
            id: player.id,
            name: player.name.clone(),
            entity,
            area_key: target_area.key.clone(),
            victories: player.victories.clone(),
        };

        if target_area.flags.victory && !new_player.victories.contains(&target_area.key) {
            new_player.victories.push(target_area.key.clone());

            self.send_server_announcement(format!(
                "{} just completed {}!",
                player.name, target_area.full_name
            ));
        }

        let player_arcswap = self.get_player_arcswap(req.player_id)?;
        player_arcswap.store(Arc::new(new_player));

        if should_close {
            self.close_area(&player_area.key);
        }

        drop(player_area);

        let (render, pos) = target_area
            .world
            .query_one_mut::<(&RenderReceiver, &mut Position)>(entity)
            .unwrap();

        pos.0 = target_pos;

        self.transfer_queue.swap_remove(
            self.transfer_queue
                .iter()
                .position(|&hash| hash == req.player_id)
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
        let player = self.get_player(req.player_id)?;

        let area = self.get_or_create_area(&player.area_key)?;
        let mut area = area.lock().await;

        let area_spawn_pos = area.spawn_pos;

        let pos = area.world.query_one_mut::<&mut Position>(player.entity)?;

        pos.0 = req.target_pos.unwrap_or(area_spawn_pos);

        Ok(())
    }

    pub async fn update_player_input(&mut self, player_id: u64, input: Vec2) -> Result<()> {
        let player = self.get_player(player_id)?;

        let area = self.get_or_create_area(&player.area_key)?;
        let mut area = area.lock().await;

        area.update_player_input(player.entity, input);

        Ok(())
    }

    pub fn get_player_arcswap(&self, player_id: u64) -> Result<&ArcSwap<Player>> {
        let player = self
            .players
            .get(&player_id)
            .ok_or(anyhow!("Player with ID @{player_id} not found"))?;

        Ok(player)
    }

    pub fn get_player(&self, player_id: u64) -> Result<Guard<Arc<Player>>> {
        let player = self.get_player_arcswap(player_id)?.load();

        Ok(player)
    }

    pub fn get_player_by_name(&self, name: &str) -> Result<Guard<Arc<Player>>> {
        self.players
            .values()
            .find_map(|player| {
                let player = player.load();

                if player.name == name {
                    Some(player)
                } else {
                    None
                }
            })
            .ok_or(anyhow!("Player '{name}' not found"))
    }

    fn send_server_announcement(&self, message: String) {
        let chat_broadcast = ChatRequest::new(
            message,
            String::new(),
            u64::MAX,
            ChatMessageType::ServerAnnouncement,
            None,
        );
        let _ = self.chat_tx.send(chat_broadcast);
    }
}

pub struct Player {
    pub id: u64,
    pub name: String,
    pub entity: Entity,
    pub area_key: AreaKey,
    pub victories: Vec<AreaKey>,
}

impl Player {
    pub fn new(id: u64, name: String, entity: Entity, area_key: AreaKey) -> Self {
        Self {
            id,
            entity,
            area_key,
            name,
            victories: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransferRequest {
    pub player_id: u64,
    pub target: TransferTarget,
    pub target_pos: Option<Vec2>,
}

#[derive(Clone, Debug)]
pub enum TransferTarget {
    Area(AreaKey),
    MapStart(String),
    Spawn,
}
