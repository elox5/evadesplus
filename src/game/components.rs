use crate::physics::vec2::Vec2;
use serde::Deserialize;
use wtransport::Connection;

pub struct PlayerId(pub u64);
pub struct Timer(pub f32);

pub struct Hero;
pub struct Enemy;

pub struct Downed;

pub struct Bounded;
pub struct BounceOffBounds;

pub struct CrossingPortal;

pub struct RenderReceiver {
    pub connection: Connection,
}

pub struct Position(pub Vec2);
pub struct Velocity(pub Vec2);

pub struct Speed(pub f32);
pub struct Direction(pub Vec2);

pub struct Size(pub f32);

#[derive(Clone, Default, Deserialize)]
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

    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        if hex.chars().count() < 8 {
            return Self { r, g, b, a: 255 };
        }

        let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);

        Self { r, g, b, a }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) + ((self.r as u32) << 16) + ((self.g as u32) << 8) + (self.b as u32)
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<&str> for Color {
    fn from(hex: &str) -> Self {
        Self::from_hex(hex)
    }
}

impl From<String> for Color {
    fn from(hex: String) -> Self {
        Self::from_hex(&hex)
    }
}
