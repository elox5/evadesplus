use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{interval, Instant},
};
use wtransport::Connection;

pub struct Player {
    pub id: u64,

    x: f32,
    y: f32,
    pub r: f32,
    pub color: String,

    pub dir_x: f32,
    pub dir_y: f32,

    pub speed: f32,

    connection: Connection,
}

impl Player {
    pub fn new(id: u64, connection: Connection) -> Self {
        let color = format!("#{:06x}", rand::random::<u32>());

        Self {
            id,
            x: 0.0,
            y: 0.0,
            r: 0.5,
            color,
            dir_x: 0.0,
            dir_y: 0.0,
            speed: 17.0,
            connection,
        }
    }
}

pub struct World {
    pub players: Vec<Player>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }

    pub fn create_player(&mut self, id: u64, connection: Connection) -> &Player {
        let player = Player::new(id, connection);
        println!("Player (id: {:X}) created", &player.id);
        self.players.push(player);

        return &self.players[self.players.len() - 1];
    }

    pub fn update_player_input(&mut self, id: u64, vx: f32, vy: f32) {
        let player = self.players.iter_mut().find(|player| player.id == id);

        if let Some(player) = player {
            player.dir_x = vx;
            player.dir_y = vy;
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for player in &mut self.players {
            player.x += player.dir_x * player.speed * delta_time;
            player.y += player.dir_y * player.speed * delta_time;

            let x_bytes = player.x.to_le_bytes();
            let y_bytes = player.y.to_le_bytes();

            let data = vec![
                x_bytes[0], x_bytes[1], x_bytes[2], x_bytes[3], y_bytes[0], y_bytes[1], y_bytes[2],
                y_bytes[3],
            ];
            let _ = player.connection.send_datagram(data);

            println!(
                "Player (id: {:X}) moved to ({}, {})",
                player.id, player.x, player.y
            );
        }
    }

    pub fn start_update_loop(world: Arc<Mutex<World>>) {
        tokio::spawn(async move {
            let mut last_time = Instant::now();

            let mut interval = interval(Duration::from_millis(16));

            loop {
                {
                    let mut world = world.lock().await;
                    world.update(last_time.elapsed().as_secs_f32());
                }

                last_time = Instant::now();
                interval.tick().await;
            }
        });
    }
}
