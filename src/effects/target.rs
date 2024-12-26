use super::core_types::{EffectMain, UpdateEffects};
use std::{
    ops::{AddAssign, MulAssign},
    sync::{mpsc, Weak},
};

pub trait EffectTarget
where
    Self: Sized,
    Self::EffectValue: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    type EffectValue;

    fn apply(&mut self, effect: EffectMain<Self::EffectValue>)
        -> Weak<mpsc::Sender<UpdateEffects>>;
}
