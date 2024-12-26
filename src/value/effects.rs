use super::Value;
use crate::effects::{
    core_types::{EffectMain, UpdateEffects},
    target::EffectTarget,
};
use std::{
    ops::{Add, Mul},
    sync::mpsc,
};

impl<T, TAdd, TMul> EffectTarget for Value<T, TAdd, TMul>
where
    T: Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync,
    TMul: Copy + Send + Sync,
{
    type EffectValue = T;
    type EffectAdd = TAdd;
    type EffectMul = TMul;

    fn apply(&mut self, effect: EffectMain<T, TAdd, TMul>) -> mpsc::Sender<UpdateEffects> {
        let mut move_back = Vec::new();
        let mut insert_at = 0;
        for (i, other_effect) in self.effects.iter().enumerate().rev() {
            if effect.priority < other_effect.priority
                || (effect.priority == other_effect.priority
                    && effect.priority.group == other_effect.priority.group)
            {
                insert_at = i + 1;
                break;
            } else if effect.priority > other_effect.priority {
                move_back.push(i - move_back.len());
            }
        }
        move_back.into_iter().rev().for_each(|i| {
            let removed = self.effects.remove(i);
            self.effects.push(removed);
        });
        self.effects.insert(insert_at, effect);
        self.recalculate();
        self.tx.clone()
    }
}
