use std::ops::{Add, Mul};

#[derive(Clone, Copy)]
pub enum EffectAction<T, TAdd = T, TMul = T>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync,
    TMul: Copy + Send + Sync,
{
    None,
    Overwrite(T),
    Add(TAdd),
    Multiply(TMul),
}

impl<T, TAdd, TMul> EffectAction<T, TAdd, TMul>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync,
    TMul: Copy + Send + Sync,
{
    pub fn apply_to(&self, value: &mut T) {
        match *self {
            EffectAction::None => {}
            EffectAction::Overwrite(applied) => *value = applied,
            EffectAction::Add(applied) => *value = *value + applied,
            EffectAction::Multiply(applied) => *value = *value * applied,
        }
    }
}
