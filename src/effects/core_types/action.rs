use std::ops::{AddAssign, MulAssign};

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
    pub fn apply_to(&self, value: &mut T) {
        match *self {
            EffectAction::Overwrite(applied) => *value = applied,
            EffectAction::Add(applied) => *value += applied,
            EffectAction::Multiply(applied) => *value *= applied,
        }
    }
}
