pub struct Player {
    pub id: u64,

    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub color: String,
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
}
