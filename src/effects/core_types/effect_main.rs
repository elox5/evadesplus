use super::{EffectAction, EffectId, EffectPriority};
use std::{
    ops::{AddAssign, MulAssign},
    sync::{Arc, Weak},
};

#[derive(Clone)]
pub struct EffectMain<T>
where
    T: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    pub id: EffectId,
    pub priority: EffectPriority,
    pub action: Weak<EffectAction<T>>,
}

impl<T> EffectMain<T>
where
    T: Clone + Copy + Send + Sync + AddAssign + MulAssign,
{
    pub fn new(id: EffectId, priority: EffectPriority, action: &Arc<EffectAction<T>>) -> Self {
        Self {
            id,
            priority,
            action: Arc::downgrade(action),
        }
    }
}
