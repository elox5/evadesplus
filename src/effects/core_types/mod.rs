pub mod action;
pub mod effect_main;
pub mod priority;

pub use action::EffectAction;
pub use effect_main::EffectMain;
pub use priority::EffectPriority;

pub struct UpdateEffects;

pub type EffectId = u16;
pub type PriorityGroup = u16;
