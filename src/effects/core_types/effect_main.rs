use arc_swap::ArcSwap;

use super::{EffectAction, EffectId, EffectPriority};
use std::{
    ops::{Add, Mul},
    sync::{Arc, Weak},
};

#[derive(Clone)]
pub struct EffectMain<T, TAdd = T, TMul = T>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync + Mul<f32, Output = TAdd>,
    TMul: Copy + Send + Sync + Mul<f32, Output = TMul>,
{
    pub id: EffectId,
    pub priority: EffectPriority,
    pub action: Weak<ArcSwap<(bool, EffectAction<T, TAdd, TMul>)>>,
}

impl<T, TAdd, TMul> EffectMain<T, TAdd, TMul>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync + Mul<f32, Output = TAdd>,
    TMul: Copy + Send + Sync + Mul<f32, Output = TMul>,
{
    pub fn new(
        id: EffectId,
        priority: EffectPriority,
        action: &Arc<ArcSwap<(bool, EffectAction<T, TAdd, TMul>)>>,
    ) -> Self {
        Self {
            id,
            priority,
            action: Arc::downgrade(action),
        }
    }
}
