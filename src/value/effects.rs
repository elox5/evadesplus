use std::{
    ops::{AddAssign, MulAssign},
    sync::{mpsc, Arc, Weak},
};

use crate::effects::{
    core_types::{EffectMain, UpdateEffects},
    target::EffectTarget,
};

use super::Value;

impl<T> EffectTarget for Value<T>
where
    T: Copy + Send + Sync + AddAssign + MulAssign,
{
    type EffectValue = T;

    fn apply(&mut self, effect: EffectMain<T>) -> Weak<mpsc::Sender<UpdateEffects>> {
        let copy = self.find_copy(&effect);
        if let Some(copy) = copy {
            let priority = unsafe { self.effects.get_unchecked(copy) }.priority;
            if effect.priority >= priority {
                self.effects.remove(copy);
            } else {
                return Arc::downgrade(&self.tx);
            }
        }
        let mut move_back = Vec::new();
        for (i, other_effect) in self.effects.iter().enumerate() {
            if effect.priority < other_effect.priority {
                self.effects.insert(i + 1, effect);
                return Arc::downgrade(&self.tx);
            } else if effect.priority > other_effect.priority {
                move_back.push(i - move_back.len());
            }
        }
        move_back.into_iter().for_each(|i| {
            let removed = self.effects.remove(i);
            self.effects.push(removed);
        });
        self.effects.insert(0, effect);
        self.value = self.base;
        self.recalculate();
        Arc::downgrade(&self.tx)
    }
}
