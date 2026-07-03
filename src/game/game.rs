use super::{
    area::{Area, AreaKey},
    components::{CrossingPortal, Downed, Position},
    map_table::try_get_map,
    systems::*,
};
use crate::{
    config::CONFIG,
    game::{
        components::Timer,
        player::PlayerId,
        timer_sync_packet::TimerSyncPacket,
        transfer_request::{TransferRequest, TransferTarget},
    },
    logger::Logger,
    networking::{
        leaderboard::AreaInfo,
        rendering::{AreaDefinitionMessage, AreaRenderMessage},
    },
    physics::vec2::Vec2,
};
use anyhow::Result;
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

pub struct Game {
    areas: HashMap<AreaKey, Arc<Mutex<Area>>>,

    spawn_area_key: AreaKey,

    output_tx: broadcast::Sender<GameOutputMessage>,

    transfer_tx: mpsc::Sender<TransferRequest>,
    transfer_queue: Vec<PlayerId>,

    render_tx: mpsc::Sender<AreaRenderMessage>,

    timer_sync_tx: broadcast::Sender<TimerSyncPacket>,
    pub timer_sync_rx: broadcast::Receiver<TimerSyncPacket>,

    frame_duration: Duration,
}

impl Game {
    pub fn new() -> GameHandle {
        let (transfer_tx, mut transfer_rx) = mpsc::channel::<TransferRequest>(8);
        let (render_tx, mut render_rx) = mpsc::channel::<AreaRenderMessage>(64);
        let (timer_sync_tx, timer_sync_rx) = broadcast::channel(8);

        let (output_tx, output_rx) = broadcast::channel(64);

        let config = &CONFIG.game;

        let frame_duration = Duration::from_secs_f32(1.0 / config.simulation_framerate);

        let spawn_map_id = config
            .spawn_map
            .as_ref()
            .expect("Spawn map not defined in config file");

        let spawn_area_key = try_get_map(&spawn_map_id)
            .expect("Could not find start map")
            .get_start_area()
            .key
            .clone();

        let game = Game {
            areas: HashMap::new(),
            spawn_area_key,
            output_tx: output_tx.clone(),
            transfer_tx: transfer_tx.clone(),
            transfer_queue: Vec::new(),
            timer_sync_tx,
            timer_sync_rx,
            render_tx,
            frame_duration,
        };

        let arc = Arc::new(Mutex::new(game));
        let transfer_arc = arc.clone();
        let handle_arc = arc.clone();

        tokio::spawn(async move {
            while let Some(req) = transfer_rx.recv().await {
                let mut game = transfer_arc.lock().await;

                let _ = game.transfer_hero(req).await;
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = render_rx.recv().await {
                let _ = output_tx.send(GameOutputMessage::AreaRender(msg));
            }
        });

        GameHandle::new(handle_arc, output_rx)
    }

    async fn handle_spawn_request(&mut self) -> GameSpawnResult {
        let area = self.get_spawn_area();
        let mut area = area.lock().await;

        let entity = area.spawn_player();

        Logger::info(format!("Spawning hero..."));

        let player_id = PlayerId {
            entity,
            area: area.key.clone(),
        };

        let area_definition = AreaDefinitionMessage {
            id: player_id.clone(),
            data: area.definition_packet(),
        };

        let _ = self
            .output_tx
            .send(GameOutputMessage::AreaDefinition(area_definition));

        GameSpawnResult {
            player_id,
            area_info: AreaInfo::from_area(&area),
        }
    }

    fn try_create_area(&mut self, key: &AreaKey) -> Result<Arc<Mutex<Area>>> {
        let map_id = key.map_id();

        let map =
            try_get_map(map_id).ok_or_else(|| anyhow::anyhow!("Map '{}' not found", map_id))?;

        let template = map
            .try_get_area(key.order() as usize)
            .ok_or_else(|| anyhow::anyhow!("Area '{}' not found", key))?;

        let area = Area::new(
            template,
            self.transfer_tx.clone(),
            self.render_tx.clone(),
            self.timer_sync_tx.clone(),
        );

        let area = Arc::new(Mutex::new(area));
        Self::start_update_loop(area.clone(), self.frame_duration);
        self.areas.insert(key.clone(), area.clone());

        Logger::debug(format!(
            "Area {} opened. Loaded areas: {:?}",
            key,
            self.areas.keys().collect::<Vec<_>>()
        ));

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

        Logger::debug(format!(
            "Area {} closed. Loaded areas: {:?}",
            key,
            self.areas.keys().collect::<Vec<_>>()
        ));
    }

    fn get_spawn_area(&mut self) -> Arc<Mutex<Area>> {
        let spawn_area_key = self.spawn_area_key.clone();
        self.get_or_create_area(&spawn_area_key)
            .unwrap_or_else(|_| panic!("Start area '{spawn_area_key}' not found"))
    }

    async fn update_area(area: &mut Area, delta_time: f32) {
        area.frame_count += 1;
        area.time += delta_time;
        area.delta_time = delta_time;

        system_increment_timer(area);

        system_update_velocity(area);
        system_update_position(area);
        system_bounds_check(area);

        system_inner_wall_collision(area);
        system_safe_zone_collision(area);
        system_portals(area).await;

        system_hero_collision(area);
        system_enemy_collision(area);

        system_render(area);

        if area.frame_count % 300 == 0 {
            system_sync_timers(area);
        }
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

                    if let Some(packet) = &&area.render_packet {
                        let _ = area
                            .render_tx
                            .send(AreaRenderMessage {
                                key: area.key.clone(),
                                packet: packet.clone(),
                            })
                            .await;
                    }
                }

                last_time = Instant::now();
                interval.tick().await;
            }
        });

        area.try_lock().unwrap().loop_handle = Some(handle.abort_handle());
    }

    pub async fn despawn_hero(&mut self, player_id: PlayerId) -> Result<()> {
        let area_arc = self.get_or_create_area(&player_id.area)?;
        let mut area = area_arc.lock().await;

        let (_, should_close) = area.despawn_player(player_id.entity);

        // let _ = self
        //     .leaderboard_tx
        //     .send(LeaderboardUpdate::remove(player_id));

        // TODO: LB FIX

        Logger::info(format!("Despawning player @{}...", player_id));

        if should_close {
            self.close_area(&area.key);
        }

        Ok(())
    }

    pub async fn reset_hero(&mut self, player: PlayerId) -> Result<()> {
        let req = TransferRequest {
            player: player.clone(),
            target: TransferTarget::Spawn,
            target_pos: None,
        };

        self.transfer_hero(req).await?;

        // let _ = self
        //     .leaderboard_tx
        //     .send(LeaderboardUpdate::set_downed(player, false));

        // TODO: LB FIX

        let area = self.get_or_create_area(&player.area)?;
        let mut area = area.lock().await;

        let _ = area.world.remove_one::<Downed>(player.entity);

        if let Ok(timer) = area.world.query_one_mut::<&mut Timer>(player.entity) {
            timer.0 = 0.0;
        }

        Ok(())
    }

    pub async fn transfer_hero(&mut self, req: TransferRequest) -> Result<()> {
        if !self.transfer_queue.contains(&req.player) {
            self.transfer_queue.push(req.player.clone());
        }

        let target_key = match req.target {
            TransferTarget::Spawn => self.spawn_area_key.clone(),
            TransferTarget::MapStart(ref map_id) => try_get_map(&map_id)
                .ok_or_else(|| anyhow::anyhow!("Map '{}' not found", map_id))?
                .get_start_area()
                .key
                .clone(),
            TransferTarget::Area(ref key) => key.clone(),
        };

        if target_key == req.player.area {
            return self.move_hero_across_area(req).await;
        }

        let target_area_arc = self.get_or_create_area(&target_key)?;

        let player_area = self.get_or_create_area(&req.player.area)?;

        let (mut player_area, mut target_area) = join!(player_area.lock(), target_area_arc.lock());

        let (taken_entity, should_close) = player_area.despawn_player(req.player.entity);
        let entity = taken_entity?;
        let entity = target_area.world.spawn(entity);

        let _ = target_area.world.remove_one::<CrossingPortal>(entity);

        // let _ = self.leaderboard_tx.send(LeaderboardUpdate::transfer(
        //     req.player_id,
        //     AreaInfo::new(&target_area),
        // ));

        // TODO: LB FIX

        let target_pos = match req.target_pos {
            Some(target_pos) => {
                let target_x = target_pos.x.resolve(&target_area.bounds);
                let target_y = target_pos.y.resolve(&target_area.bounds);

                Vec2::new(target_x, target_y)
            }
            None => target_area.spawn_pos,
        };

        // if target_area.flags.victory && !new_player.victories.contains(&target_area.key) {
        //     new_player.victories.push(target_area.key.clone());

        // let world = &mut target_area.world;
        // let timer = world.query_one_mut::<&mut Timer>(entity).ok();

        // if let Some(timer) = timer {
        //     let minutes = timer.0 / 60.0;
        //     let seconds = (timer.0.floor() as u32) % 60;

        //     let announcement_name = match &target_area.route_name {
        //         Some(route) => route,
        //         None => match &target_area.flags.final_victory {
        //             true => &target_area.map_name,
        //             false => &target_area.full_name,
        //         },
        //     };

        //     self.send_server_announcement(format!(
        //         "{} just completed {} in {:02.0}:{:02.0}!",
        //         player.name, announcement_name, minutes, seconds
        //     ));
        // } else {
        //     Logger::error("Expected Timer component on hero when transferring to victory area");
        // }

        // TODO: CHAT FIX
        // }

        if should_close {
            self.close_area(&player_area.key);
        }

        drop(player_area);

        let pos = target_area
            .world
            .query_one_mut::<&mut Position>(entity)
            .unwrap();

        pos.0 = target_pos;

        self.transfer_queue.swap_remove(
            self.transfer_queue
                .iter()
                .position(|id| *id == req.player)
                .unwrap(),
        );

        // let mut response_stream = render.connection.open_uni().await?.await?;
        // response_stream
        //     .write_all(&target_area.definition_packet())
        //     .await?;
        // response_stream.finish().await?;

        Ok(())
    }

    pub async fn move_hero_across_area(&mut self, req: TransferRequest) -> Result<()> {
        let area = self.get_or_create_area(&req.player.area)?;
        let mut area = area.lock().await;
        let bounds = area.bounds.clone();

        let area_spawn_pos = area.spawn_pos;

        let pos = area
            .world
            .query_one_mut::<&mut Position>(req.player.entity)?;

        let target_pos = match req.target_pos {
            Some(target_pos) => {
                let target_x = target_pos.x.resolve(&bounds);
                let target_y = target_pos.y.resolve(&bounds);

                Vec2::new(target_x, target_y)
            }
            None => area_spawn_pos,
        };

        pos.0 = target_pos;

        Ok(())
    }

    pub async fn update_player_input(&mut self, player_id: PlayerId, input: Vec2) -> Result<()> {
        let area = self.get_or_create_area(&player_id.area)?;
        let mut area = area.lock().await;

        area.update_player_input(player_id.entity, input);

        Ok(())
    }
}

pub struct GameHandle {
    game: Arc<Mutex<Game>>,
    pub output_rx: broadcast::Receiver<GameOutputMessage>,
}

impl GameHandle {
    fn new(game: Arc<Mutex<Game>>, output_rx: broadcast::Receiver<GameOutputMessage>) -> Self {
        Self { game, output_rx }
    }

    pub async fn send_spawn_request(&self) -> GameSpawnResult {
        let mut game = self.game.lock().await;
        game.handle_spawn_request().await
    }
}

impl Clone for GameHandle {
    fn clone(&self) -> Self {
        Self {
            game: self.game.clone(),
            output_rx: self.output_rx.resubscribe(),
        }
    }
}

#[derive(Clone)]
pub enum GameOutputMessage {
    AreaRender(AreaRenderMessage),
    AreaDefinition(AreaDefinitionMessage),
}

pub struct GameSpawnResult {
    pub player_id: PlayerId,
    pub area_info: AreaInfo,
}
