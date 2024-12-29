use std::ops::{Add, Mul};

#[derive(Clone, Copy)]
pub enum EffectAction<T, TAdd = T, TMul = T>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync + Mul<f32, Output = TAdd>,
    TMul: Copy + Send + Sync + Mul<f32, Output = TMul>,
{
    None,
    Overwrite(T),
    Add(TAdd),
    Multiply(TMul),
}

impl<T, TAdd, TMul> EffectAction<T, TAdd, TMul>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync + Mul<f32, Output = TAdd>,
    TMul: Copy + Send + Sync + Mul<f32, Output = TMul>,
{
    pub fn apply_to(&self, value: &mut T, receptivity: f32) {
        match *self {
            EffectAction::None => {}
            EffectAction::Overwrite(applied) => {
                if receptivity < 0.5 {
                    *value = applied
                }
            }
            EffectAction::Add(applied) => *value = *value + applied * receptivity,
            EffectAction::Multiply(applied) => *value = *value * (applied * receptivity),
        }
    }

    pub fn apply_without_receptivity_to(&self, value: &mut T) {
        match *self {
            EffectAction::None => {}
            EffectAction::Overwrite(applied) => *value = applied,
            EffectAction::Add(applied) => *value = *value + applied,
            EffectAction::Multiply(applied) => *value = *value * applied,
        }
    }
}
