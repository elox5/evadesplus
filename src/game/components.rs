use crate::physics::vec2::Vec2;
use wtransport::Connection;

pub struct Hero;
pub struct Enemy;

pub struct Player {
    pub name: String,
    pub connection: Connection,
}

pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

pub struct Speed(pub f32);
pub struct Direction(pub Vec2);

pub struct Size(pub f32);

#[derive(Clone, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) + ((self.r as u32) << 16) + ((self.g as u32) << 8) + (self.b as u32)
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub struct Bounded;
pub struct BounceOffBounds;
