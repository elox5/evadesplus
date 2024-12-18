use super::{area::Area, systems::*, templates::AreaTemplate};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{interval, Instant},
};

pub struct Game {
    pub areas: Vec<Arc<Mutex<Area>>>,
}

impl Game {
    pub fn new() -> Self {
        Self { areas: Vec::new() }
    }

    pub fn create_area(&mut self, template: AreaTemplate) {
        let area = Area::from_template(template);
        let area = Arc::new(Mutex::new(area));
        let _ = Self::start_update_loop(area.clone());
        self.areas.push(area);
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
}
