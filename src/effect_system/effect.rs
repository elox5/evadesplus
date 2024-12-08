use std::{
    ops::{AddAssign, MulAssign},
    sync::{Arc, Weak},
};

use crate::effect_system::{priority::EffectPriority, target::EffectId};

pub struct UpdateEffects;

pub trait Effect {
    fn clear(self);
}

#[derive(Clone)]
pub struct EffectMain<T>
where
    T: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    pub id: EffectId,
    pub priority: EffectPriority,
    pub effect: Weak<EffectAction<T>>,
}

impl<T> EffectMain<T>
where
    T: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    pub fn new(id: EffectId, priority: EffectPriority, effect: &Arc<EffectAction<T>>) -> Self {
        Self {
            id,
            priority,
            effect: Arc::downgrade(effect),
        }
    }
}

#[derive(Clone, Copy)]
pub enum EffectAction<T>
where
    T: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    Overwrite(T),
    Add(T),
    Multiply(T),
}

impl<T> EffectAction<T>
where
    T: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    pub fn apply(&self, value: &mut T) {
        match *self {
            EffectAction::Overwrite(applied) => *value = applied,
            EffectAction::Add(applied) => *value += applied,
            EffectAction::Multiply(applied) => *value *= applied,
        }
    }
}
