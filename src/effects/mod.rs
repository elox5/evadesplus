pub mod core_types;
pub mod string_tables;
pub mod target;
pub mod timed_effect;
pub mod togglable_effect;

use core_types::{EffectAction, EffectPriority};
use std::{sync::Weak, time::Duration};
use string_tables::ID_TABLE;
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
        target_list: &mut Vec<&mut T>,
        id: &'static str,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ignore_receptivity: bool,
    ) -> Self {
        Self::Togglable(Some(TogglableEffect::apply(
            target_list,
            ID_TABLE.get(id),
            priority,
            action,
            ignore_receptivity,
        )))
    }

    pub fn apply_toggle_from_string(
        target_list: &mut Vec<&mut T>,
        id: String,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ignore_receptivity: bool,
    ) -> Self {
        Self::Togglable(Some(TogglableEffect::apply(
            target_list,
            ID_TABLE.get_or_insert(id),
            priority,
            action,
            ignore_receptivity,
        )))
    }

    pub fn apply_timed(
        target_list: &mut Vec<&mut T>,
        id: &'static str,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ignore_receptivity: bool,
        duration: Duration,
    ) -> Self {
        Self::Timed(TimedEffect::apply(
            target_list,
            ID_TABLE.get(id),
            priority,
            action,
            ignore_receptivity,
            duration,
        ))
    }

    pub fn apply_timed_from_string(
        target_list: &mut Vec<&mut T>,
        id: String,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ignore_receptivity: bool,
        duration: Duration,
    ) -> Self {
        Self::Timed(TimedEffect::apply(
            target_list,
            ID_TABLE.get_or_insert(id),
            priority,
            action,
            ignore_receptivity,
            duration,
        ))
    }

    pub fn update(
        &self,
        action: (
            bool,
            EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ),
    ) {
        match self {
            Effect::Togglable(effect) => {
                if let Some(effect) = effect {
                    effect.update(action);
                }
            }
            Effect::Timed(effect) => {
                if let Some(effect) = effect.upgrade() {
                    effect.update(action);
                }
            }
        }
    }

    pub fn get(
        &self,
    ) -> Option<(
        bool,
        EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
    )> {
        match self {
            Effect::Togglable(effect) => {
                if let Some(effect) = effect {
                    return Some(effect.get());
                }
            }
            Effect::Timed(effect) => {
                if let Some(effect) = effect.upgrade() {
                    return Some(effect.get());
                }
            }
        }
        None
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
