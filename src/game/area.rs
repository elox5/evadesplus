use super::{
    components::{
        BounceOffBounds, Bounded, Color, Direction, Enemy, Hero, Player, Position, Size, Speed,
        Velocity,
    },
    templates::{AreaTemplate, EnemyGroup},
};
use crate::{
    networking::rendering::RenderPacket,
    physics::{rect::Rect, vec2::Vec2},
};
use hecs::{Entity, World};
use tokio::task::AbortHandle;
use wtransport::Connection;

pub struct Area {
    pub id: String,
    pub name: String,
    pub background_color: Color,

    pub world: World,

    pub bounds: Rect,
    pub inner_walls: Vec<Rect>,

    pub time: f32,
    pub delta_time: f32,

    pub render_packet: Option<RenderPacket>,

    pub loop_handle: Option<AbortHandle>,
}

impl Area {
    pub fn new(
        id: String,
        name: String,
        background_color: Color,
        width: f32,
        height: f32,
        inner_walls: Vec<Rect>,
    ) -> Self {
        Self {
            name,
            id,
            background_color,
            world: World::new(),
            bounds: Rect::new(0.0, 0.0, width, height),
            inner_walls,
            time: 0.0,
            delta_time: 0.0,
            render_packet: None,
            loop_handle: None,
        }
    }

    pub fn from_template(template: AreaTemplate) -> Self {
        let mut area = Self::new(
            template.id,
            template.name,
            template.background_color,
            template.width,
            template.height,
            template.inner_walls,
        );

        for group in template.enemy_groups {
            area.spawn_enemy_group(&group);
        }

        area
    }

    pub fn close(&mut self) {
        if let Some(handle) = self.loop_handle.take() {
            handle.abort();
        }
    }

    pub fn spawn_enemy_group(&mut self, group: &EnemyGroup) {
        let enemies = (0..group.count).map(|_| {
            let pos = Position(self.bounds.random_inside());
            let vel = Velocity(Vec2::ZERO);
            let dir = Direction(Vec2::random_unit());
            let speed = Speed(group.speed);
            let size = Size(group.size);
            let color = group.color.clone();

            (
                Enemy,
                pos,
                vel,
                dir,
                speed,
                size,
                color,
                Bounded,
                BounceOffBounds,
            )
        });

        self.world.spawn_batch(enemies);
    }

    pub fn spawn_hero(&mut self, name: &str, connection: Connection) -> Entity {
        let player = Player {
            connection,
            name: name.to_owned(),
        };

        let pos = Position(self.bounds.center());
        let vel = Velocity(Vec2::ZERO);
        let speed = Speed(17.0);
        let dir = Direction(Vec2::ZERO);

        let size = Size(1.0);
        let color = Color::rgb(rand::random(), rand::random(), rand::random());

        self.world
            .spawn((player, Hero, pos, vel, speed, dir, size, color, Bounded))
    }

    pub fn update_hero_dir(&mut self, entity: Entity, new_dir: Vec2) {
        let dir = self.world.query_one_mut::<&mut Direction>(entity).unwrap();
        dir.0 = new_dir;
    }
}
