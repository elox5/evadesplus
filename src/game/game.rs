use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Player {
    pub id: u64,

    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub color: String,

    pub vx: f32,
    pub vy: f32,
}

impl Player {
    pub fn new() -> Self {
        let id = rand::random();

        let color = format!("#{:06x}", rand::random::<u32>());

        Self {
            id,
            x: 0.0,
            y: 0.0,
            r: 0.5,
            color,
            vx: 0.0,
            vy: 0.0,
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

    pub fn create_player(&mut self) -> &Player {
        let player = Player::new();
        println!("Player (id: {:X}) created", &player.id);
        self.players.push(player);

        return &self.players[self.players.len() - 1];
    }

    pub fn update_player_input(&mut self, id: u64, vx: f32, vy: f32) {
        let player = self.players.iter_mut().find(|player| player.id == id);

        if let Some(player) = player {
            player.vx = vx;
            player.vy = vy;
        }
    }

    pub fn update(&mut self) {
        for player in &mut self.players {
            player.x += player.vx;
            player.y += player.vy;

            println!(
                "Player (id: {:X}) moved to ({}, {})",
                player.id, player.x, player.y
            );
        }
    }

    pub fn start_update_loop(world: Arc<Mutex<World>>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;

                let mut world = world.lock().await;
                world.update();
            }
        });
    }
}
