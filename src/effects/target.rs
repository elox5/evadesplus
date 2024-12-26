use super::core_types::{EffectMain, UpdateEffects};
use std::{
    ops::{Add, Mul},
    sync::{mpsc, Weak},
};

pub trait EffectTarget
where
    Self: Sized,
    Self::EffectValue: Copy
        + Send
        + Sync
        + Add<Self::EffectAdd, Output = Self::EffectValue>
        + Mul<Self::EffectMul, Output = Self::EffectValue>,
    Self::EffectAdd: Copy + Send + Sync,
    Self::EffectMul: Copy + Send + Sync,
{
    type EffectValue;
    type EffectAdd;
    type EffectMul;

    fn apply(
        &mut self,
        effect: EffectMain<Self::EffectValue, Self::EffectAdd, Self::EffectMul>,
    ) -> Weak<mpsc::Sender<UpdateEffects>>;
}
