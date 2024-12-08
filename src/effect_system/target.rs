use std::{
    ops::{AddAssign, MulAssign},
    sync::{mpsc, Weak},
};

use crate::effect_system::effect::{EffectMain, UpdateEffects};

pub type EffectId = u32;

pub trait EffectTarget
where
    Self: Sized,
    Self::EffectValue: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    type EffectValue;

    fn apply(&mut self, effect: EffectMain<Self::EffectValue>)
        -> Weak<mpsc::Sender<UpdateEffects>>;
}
