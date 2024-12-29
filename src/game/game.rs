use super::{
    area::Area,
    components::{self, Hero, Position},
    data::MapData,
    systems::*,
    templates::MapTemplate,
};
use crate::physics::vec2::Vec2;
use anyhow::Result;
use arc_swap::{ArcSwap, Guard};
use hecs::Entity;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
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

    transfer_tx: mpsc::Sender<(Entity, String, Vec2)>,
}

impl Game {
    pub fn new(maps: Vec<MapData>, start_area_id: &str) -> Arc<Mutex<Self>> {
        let (tx, mut rx) = mpsc::channel::<(Entity, String, Vec2)>(8);

        let game = Game {
            maps: maps.into_iter().map(|m| m.to_template()).collect(),
            areas: Vec::new(),
            players: Vec::new(),
            start_area_id: start_area_id.to_owned(),
            transfer_tx: tx.clone(),
        };

        let arc = Arc::new(Mutex::new(game));
        let arc_clone = arc.clone();

        tokio::spawn(async move {
            while let Some((entity, target_area, target_pos)) = rx.recv().await {
                let mut game = arc_clone.lock().await;
                let _ = game.transfer_player(entity, &target_area, target_pos).await;
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
        let _ = Self::start_update_loop(area.clone());
        self.areas.push(area.clone());

        println!("Created area {}", id);
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

    pub async fn spawn_player(
        &mut self,
        name: &str,
        connection: Connection,
    ) -> Arc<ArcSwap<Player>> {
        let start_area_id = self.start_area_id.clone();

        let area_arc = self
            .get_or_create_area(&start_area_id)
            .expect(&format!("Start area '{start_area_id}' not found"));
        let mut area = area_arc.lock().await;

        let entity = area.spawn_player(name, connection);
        println!("Spawning entity: {}", entity.id());

        let player = Player::new(entity, area_arc.clone());
        let player = Arc::new(ArcSwap::new(Arc::new(player)));

        self.players.push(player.clone());

        player
    }

    pub async fn despawn_player(&mut self, entity: Entity) {
        let mut area_to_remove = None;

        if let Some(player_index) = self.players.iter().position(|p| p.load().entity == entity) {
            let player = self.players.swap_remove(player_index);
            let player = player.load();

            let mut area = player.area.lock().await;
            let _ = area.despawn_player(entity);

            let player_count = area.world.query_mut::<&Hero>().into_iter().count();
            if player_count == 0 {
                area_to_remove = Some(player.area.clone());
            }
        }

        if let Some(area) = area_to_remove {
            self.areas.retain(|a| !Arc::ptr_eq(a, &area));
            let mut area = area.lock().await;
            area.close();

            println!("Removed area {}", area.full_id);
        }
    }

    pub async fn transfer_player(
        &mut self,
        entity: Entity,
        target_area: &str,
        target_pos: Vec2,
    ) -> Result<()> {
        let target_area_arc = self.get_or_create_area(target_area)?;
        let mut target_area = target_area_arc.lock().await;

        let player_arcswap = self
            .players
            .iter_mut()
            .find(|p| p.load().entity == entity)
            .ok_or(anyhow::anyhow!("Player not found"))?;
        let player = player_arcswap.load();

        let mut area = player.area.lock().await;
        let entity = area.world.take(player.entity)?;

        let entity = target_area.world.spawn(entity);

        drop(area);

        let new_player = Player::new(entity, target_area_arc.clone());
        player_arcswap.store(Arc::new(new_player));

        let (player_component, pos) = target_area
            .world
            .query_one_mut::<(&components::Player, &mut Position)>(entity)
            .unwrap();

        pos.0 = target_pos;

        let mut response_stream = player_component.connection.open_uni().await?.await?;
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
}

pub struct Player {
    pub entity: Entity,
    pub area: Arc<Mutex<Area>>,
}

impl Player {
    pub fn new(entity: Entity, area: Arc<Mutex<Area>>) -> Self {
        Self { entity, area }
    }
}
