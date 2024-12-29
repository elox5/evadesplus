use super::core_types::{EffectMain, UpdateEffects};
use std::{
    ops::{Add, Mul},
    sync::mpsc,
};

pub trait EffectTarget
where
    Self: Sized,
    Self::EffectValue: Copy
        + Send
        + Sync
        + Add<Self::EffectAdd, Output = Self::EffectValue>
        + Mul<Self::EffectMul, Output = Self::EffectValue>,
    Self::EffectAdd: Copy + Send + Sync + Mul<f32, Output = Self::EffectAdd>,
    Self::EffectMul: Copy + Send + Sync + Mul<f32, Output = Self::EffectMul>,
{
    type EffectValue;
    type EffectAdd;
    type EffectMul;

    fn apply(
        &mut self,
        effect: EffectMain<Self::EffectValue, Self::EffectAdd, Self::EffectMul>,
    ) -> mpsc::Sender<UpdateEffects>;
}
