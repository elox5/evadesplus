pub mod action;
pub mod effect_main;
pub mod priority;

use super::target::EffectTarget;
pub use action::EffectAction;
pub use effect_main::EffectMain;
pub use priority::EffectPriority;

use arc_swap::ArcSwap;
use std::sync::Arc;

pub struct UpdateEffects;

pub type EffectId = u16;
pub type PriorityGroup = u16;
pub type EffectStore<T> = Arc<
    ArcSwap<(
        bool,
        EffectAction<
            <T as EffectTarget>::EffectValue,
            <T as EffectTarget>::EffectAdd,
            <T as EffectTarget>::EffectMul,
        >,
    )>,
>;
