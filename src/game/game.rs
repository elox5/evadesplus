use super::{
    area::Area,
    components::{Named, Position, RenderReceiver},
    data::MapData,
    systems::*,
    templates::MapTemplate,
};
use crate::{
    networking::leaderboard::{LeaderboardState, LeaderboardUpdate, LeaderboardUpdatePacket},
    physics::vec2::Vec2,
};
use anyhow::Result;
use arc_swap::{ArcSwap, Guard};
use hecs::Entity;
use std::{sync::Arc, time::Duration};
use tokio::{
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
    areas: Vec<Arc<Mutex<Area>>>,
    players: Vec<Arc<ArcSwap<Player>>>,

    start_area_id: String,

    pub leaderboard_state: LeaderboardState,

    transfer_tx: mpsc::Sender<(Entity, String, Vec2)>,
    leaderboard_tx: broadcast::Sender<LeaderboardUpdatePacket>,
    pub leaderboard_rx: broadcast::Receiver<LeaderboardUpdatePacket>,
}

impl Game {
    pub fn new(maps: Vec<MapData>, start_area_id: &str) -> Arc<Mutex<Self>> {
        let (transfer_tx, mut transfer_rx) = mpsc::channel::<(Entity, String, Vec2)>(8);
        let (leaderboard_tx, leaderboard_rx) = broadcast::channel(8);

        let game = Game {
            maps: maps.into_iter().map(|m| m.to_template()).collect(),
            areas: Vec::new(),
            players: Vec::new(),
            start_area_id: start_area_id.to_owned(),
            leaderboard_state: LeaderboardState::new(),
            transfer_tx: transfer_tx.clone(),
            leaderboard_tx,
            leaderboard_rx,
        };

        let arc = Arc::new(Mutex::new(game));
        let arc_clone = arc.clone();

        tokio::spawn(async move {
            while let Some((entity, target_area, target_pos)) = transfer_rx.recv().await {
                let mut game = arc_clone.lock().await;
                let _ = game
                    .transfer_hero(entity, &target_area, Some(target_pos))
                    .await;
            }
        });

        arc
    }

    pub fn try_create_area(&mut self, id: &str) -> Result<Arc<Mutex<Area>>> {
        let (map_id, area_id) = Self::split_id(id).ok_or(anyhow::anyhow!("Invalid id"))?;

        let map = self
            .try_get_map(map_id)
            .ok_or(anyhow::anyhow!("Map not found"))?;

        let template = map
            .get_area(area_id)
            .ok_or(anyhow::anyhow!("Area not found"))?;

        let area = Area::from_template(template, self.transfer_tx.clone());
        let area = Arc::new(Mutex::new(area));
        Self::start_update_loop(area.clone());
        self.areas.push(area.clone());

        println!("Area {} opened", id);
        Ok(area)
    }

    fn try_get_area(&self, id: &str) -> Option<Arc<Mutex<Area>>> {
        self.areas
            .iter()
            .find(|a| a.try_lock().map(|a| a.full_id == id).unwrap_or(false))
            .cloned()
    }

    pub fn get_or_create_area(&mut self, id: &str) -> Result<Arc<Mutex<Area>>> {
        if let Some(area) = self.try_get_area(id) {
            return Ok(area);
        }

        self.try_create_area(id)
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

    fn start_update_loop(area: Arc<Mutex<Area>>) {
        let area_clone = area.clone();

        let handle = tokio::spawn(async move {
            let mut last_time = Instant::now();

            let mut interval = interval(Duration::from_millis(16));

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

    pub async fn spawn_hero(&mut self, name: &str, connection: Connection) -> Arc<ArcSwap<Player>> {
        let start_area_id = self.start_area_id.clone();

        let area_arc = self
            .get_or_create_area(&start_area_id)
            .unwrap_or_else(|_| panic!("Start area '{start_area_id}' not found"));
        let mut area = area_arc.lock().await;

        let entity = area.spawn_player(name, connection);
        println!("Spawning hero (entity {})", entity.id());

        let player = Player::new(entity, area_arc.clone(), name.to_owned());
        let player = Arc::new(ArcSwap::new(Arc::new(player)));

        self.players.push(player.clone());

        let add_entry = LeaderboardUpdatePacket::add(
            entity,
            area.full_id.clone(),
            name.to_owned(),
            area.map_name.clone(),
            area.area_name.clone(),
            area.order,
        );
        self.handle_leaderboard_entry(add_entry);

        player
    }

    pub async fn despawn_hero(&mut self, entity: Entity) {
        if let Some(player_index) = self.players.iter().position(|p| p.load().entity == entity) {
            println!("Despawning hero (entity {})", entity.id());

            let player = self.players.swap_remove(player_index);
            let player = player.load();

            let mut area = player.area.lock().await;
            let (_, should_close) = area.despawn_player(entity);

            let remove_entry = LeaderboardUpdatePacket::remove(entity, area.full_id.clone());
            self.handle_leaderboard_entry(remove_entry);

            if should_close {
                self.areas.retain(|a| !Arc::ptr_eq(a, &player.area));
            }
        }
    }

    pub async fn transfer_hero(
        &mut self,
        entity: Entity,
        target_area: &str,
        target_pos: Option<Vec2>,
    ) -> Result<()> {
        let target_area_arc = self.get_or_create_area(target_area)?;

        let player_arcswap = self
            .players
            .iter_mut()
            .find(|p| p.load().entity == entity)
            .unwrap();
        let player = player_arcswap.load();

        let mut area = player.area.lock().await;
        let mut target_area = target_area_arc.lock().await;

        let target_area_order = target_area.order;
        let target_area_full_id = target_area.full_id.clone();
        let target_map_name = target_area.map_name.clone();
        let target_area_name = target_area.area_name.clone();
        let target_area_spawn_pos = target_area.spawn_pos;

        let remove_entry = LeaderboardUpdatePacket::remove(entity, area.full_id.clone());

        let (entity, should_close) = area.despawn_player(player.entity);
        let entity = entity?;
        let entity = target_area.world.spawn(entity);

        drop(area);

        if should_close {
            self.areas.retain(|a| !Arc::ptr_eq(a, &player.area));
        }

        let new_player = Player::new(entity, target_area_arc.clone(), player.name.clone());
        player_arcswap.store(Arc::new(new_player));

        let (named, render, pos) = target_area
            .world
            .query_one_mut::<(&Named, &RenderReceiver, &mut Position)>(entity)
            .unwrap();

        pos.0 = target_pos.unwrap_or(target_area_spawn_pos);

        let add_entry = LeaderboardUpdatePacket::add(
            entity,
            target_area_full_id,
            named.0.clone(),
            target_map_name,
            target_area_name,
            target_area_order,
        );

        self.handle_leaderboard_entry(remove_entry);
        self.handle_leaderboard_entry(add_entry);

        let mut response_stream = render.connection.open_uni().await?.await?;
        response_stream
            .write_all(&target_area.definition_packet())
            .await?;
        response_stream.finish().await?;

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

    fn split_id(id: &str) -> Option<(&str, &str)> {
        let mut split = id.split(':');
        let map_id = split.next()?;
        let area_id = split.next()?;
        Some((map_id, area_id))
    }

    fn try_get_map(&self, map_id: &str) -> Option<&MapTemplate> {
        self.maps.iter().find(|m| m.id == map_id)
    }

    fn handle_leaderboard_entry(&mut self, entry: LeaderboardUpdatePacket) {
        let _ = self.leaderboard_tx.send(entry.clone());

        match entry.update {
            LeaderboardUpdate::Add { .. } => {
                self.leaderboard_state.add(entry);
            }
            LeaderboardUpdate::Remove => {
                self.leaderboard_state.remove(entry.get_hash());
            }
        }
    }
}

pub struct Player {
    pub entity: Entity,
    pub area: Arc<Mutex<Area>>,
    pub name: String,
}

impl Player {
    pub fn new(entity: Entity, area: Arc<Mutex<Area>>, name: String) -> Self {
        Self { entity, area, name }
    }
}
