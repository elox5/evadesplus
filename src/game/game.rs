use super::{area::Area, components::Hero, data::MapData, systems::*, templates::MapTemplate};
use crate::physics::vec2::Vec2;
use anyhow::Result;
use hecs::Entity;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{interval, Instant},
};
use wtransport::Connection;

pub struct Game {
    pub maps: Vec<MapTemplate>,
    pub areas: Vec<Arc<Mutex<Area>>>,
    pub players: Vec<Player>,

    start_area_id: String,
}

impl Game {
    pub fn new(maps: Vec<MapData>, start_area_id: &str) -> Self {
        Self {
            maps: maps.into_iter().map(|m| m.to_template()).collect(),
            areas: Vec::new(),
            players: Vec::new(),
            start_area_id: start_area_id.to_owned(),
        }
    }

    pub fn try_create_area(&mut self, id: &str) -> Result<Arc<Mutex<Area>>> {
        let (map_id, area_id) = Self::split_id(id).ok_or(anyhow::anyhow!("Invalid id"))?;

        let map = self
            .try_get_map(map_id)
            .ok_or(anyhow::anyhow!("Map not found"))?;

        let template = map
            .get_area(area_id)
            .ok_or(anyhow::anyhow!("Area not found"))?;

        let area = Area::from_template(template);
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

    fn update_area(area: &mut Area, delta_time: f32) {
        area.time += delta_time;
        area.delta_time = delta_time;

        system_update_velocity(area);
        system_update_position(area);
        system_bounds_check(area);
        system_inner_wall_collision(area);
        system_safe_zone_collision(area);

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
                    Self::update_area(&mut area, last_time.elapsed().as_secs_f32());
                }

                last_time = Instant::now();
                interval.tick().await;
            }
        });

        area.try_lock().unwrap().loop_handle = Some(handle.abort_handle());
    }

    pub async fn spawn_player(&mut self, name: &str, connection: Connection) -> &Player {
        let start_area_id = self.start_area_id.clone();

        let area_arc = self
            .get_or_create_area(&start_area_id)
            .expect(&format!("Start area '{start_area_id}' not found"));
        let mut area = area_arc.lock().await;

        let entity = area.spawn_player(name, connection);
        println!("Spawning entity: {}", entity.id());

        let player = Player::new(entity, area_arc.clone());
        self.players.push(player);

        self.players.last().unwrap()
    }

    pub async fn despawn_player(&mut self, entity: Entity) {
        let mut area_to_remove = None;

        if let Some(player_index) = self.players.iter().position(|p| p.entity == entity) {
            let player = self.players.swap_remove(player_index);
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

    pub async fn transfer_player(&mut self, entity: Entity, target_area: &str) -> Result<()> {
        let target_area_arc = self.get_or_create_area(target_area)?;
        let mut target_area = target_area_arc.lock().await;

        let player = self
            .players
            .iter_mut()
            .find(|p| p.entity == entity)
            .ok_or(anyhow::anyhow!("Player not found"))?;

        let mut area = player.area.lock().await;
        let entity = area.world.take(player.entity)?;

        target_area.world.spawn(entity);

        drop(area);

        player.area = target_area_arc.clone();

        Ok(())
    }

    pub async fn update_player_input(&mut self, entity: Entity, input: Vec2) {
        if let Some(player) = self.get_player(entity) {
            let mut area = player.area.lock().await;
            area.update_player_input(entity, input);
        }
    }

    pub fn get_player(&self, entity: Entity) -> Option<&Player> {
        self.players.iter().find(|p| p.entity == entity)
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
