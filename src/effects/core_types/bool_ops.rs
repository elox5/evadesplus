use std::ops::{Add, Mul};

#[derive(Clone, Copy)]
pub struct BoolOps(pub bool);

impl Add<BoolOps> for bool {
    type Output = bool;

    fn add(self, rhs: BoolOps) -> Self::Output {
        self | rhs.0
    }
}

impl Mul<BoolOps> for bool {
    type Output = bool;

    fn mul(self, rhs: BoolOps) -> Self::Output {
        self & rhs.0
    }
}

impl Mul<f32> for BoolOps {
    type Output = BoolOps;

    fn mul(self, _: f32) -> Self::Output {
        self
    }
}
