use super::vec2::Vec2;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.w, self.y + self.h)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x < self.x + self.w
            && point.y >= self.y
            && point.y < self.y + self.h
    }

    pub fn contains_circle(&self, center: Vec2, radius: f32) -> bool {
        let distance_abs = (center - self.center()).abs();

        if distance_abs.x > self.w / 2.0 + radius || distance_abs.y > self.h / 2.0 + radius {
            return false;
        }

        if distance_abs.x <= self.w / 2.0 || distance_abs.y <= self.h / 2.0 {
            return true;
        }

        let corner_distance_sq =
            (distance_abs - Vec2::new(self.w / 2.0, self.h / 2.0)).magnitude_sq();

        corner_distance_sq <= radius * radius
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn random_inside(&self) -> Vec2 {
        let x = self.x + self.w * (rand::random::<f32>());
        let y = self.y + self.h * (rand::random::<f32>());
        Vec2::new(x, y)
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        [
            self.x.to_le_bytes(),
            self.y.to_le_bytes(),
            self.w.to_le_bytes(),
            self.h.to_le_bytes(),
        ]
        .concat()
        .try_into()
        .unwrap()
    }
}
