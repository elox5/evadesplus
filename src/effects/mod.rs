pub mod core_types;
pub mod group_table;
pub mod id_table;
pub mod target;
pub mod timed_effect;
pub mod togglable_effect;

use core_types::{EffectAction, EffectPriority};
use std::{sync::Weak, time::Duration};
use target::EffectTarget;
use timed_effect::TimedEffect;
use togglable_effect::TogglableEffect;

pub enum Effect<T>
where
    T: EffectTarget + 'static,
{
    Togglable(Option<TogglableEffect<T>>),
    Timed(Weak<TimedEffect<T>>),
}

impl<T> Effect<T>
where
    T: EffectTarget + 'static,
{
    pub fn apply_toggle(
        id: &'static str,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        target_list: &mut Vec<&mut T>,
    ) -> Self {
        Self::Togglable(Some(TogglableEffect::apply(
            id_table::get(id),
            priority,
            action,
            target_list,
        )))
    }

    pub fn apply_toggle_from_string(
        id: String,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        target_list: &mut Vec<&mut T>,
    ) -> Self {
        Self::Togglable(Some(TogglableEffect::apply(
            id_table::get_or_insert(id),
            priority,
            action,
            target_list,
        )))
    }

    pub fn apply_timed(
        id: &'static str,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        target_list: &mut Vec<&mut T>,
        duration: Duration,
    ) -> Self {
        Self::Timed(TimedEffect::apply(
            id_table::get(id),
            priority,
            action,
            target_list,
            duration,
        ))
    }

    pub fn apply_timed_from_string(
        id: String,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        target_list: &mut Vec<&mut T>,
        duration: Duration,
    ) -> Self {
        Self::Timed(TimedEffect::apply(
            id_table::get_or_insert(id),
            priority,
            action,
            target_list,
            duration,
        ))
    }

    pub fn clear(&mut self) {
        match self {
            Effect::Togglable(effect) => *effect = None,
            Effect::Timed(effect) => {
                if let Some(effect) = effect.upgrade() {
                    effect.clear();
                }
            }
        }
    }

    pub fn marked_for_despawn(&self) -> bool {
        match self {
            Effect::Togglable(effect) => effect.is_some(),
            Effect::Timed(effect) => effect.upgrade().is_some(),
        }
    }
}
