use super::{area::Area, data::MapData, systems::*, templates::MapTemplate};
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{interval, Instant},
};

pub struct Game {
    pub maps: Vec<MapTemplate>,
    pub areas: Vec<Arc<Mutex<Area>>>,
}

impl Game {
    pub fn new(maps: Vec<MapData>) -> Self {
        Self {
            maps: maps.into_iter().map(|m| m.to_template()).collect(),
            areas: Vec::new(),
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

        Ok(area)
    }

    fn try_get_area(&self, id: &str) -> Option<Arc<Mutex<Area>>> {
        self.areas
            .iter()
            .find(|a| a.try_lock().map(|a| a.area_id == id).unwrap_or(false))
            .cloned()
    }

    pub fn get_or_create_area(&mut self, id: &str) -> Result<Arc<Mutex<Area>>> {
        if let Some(area) = self.try_get_area(id) {
            return Ok(area);
        }

        self.try_create_area(id)
    }

    fn update_area(area: &mut Area, delta_time: f32) {
        area.time += delta_time;
        area.delta_time = delta_time;

        system_update_velocity(area);
        system_update_position(area);
        system_bounds_check(area);
        system_inner_wall_collision(area);

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
