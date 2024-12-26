use std::{
    ops::{AddAssign, MulAssign},
    sync::{mpsc, Arc},
};

use crate::effects::core_types::{EffectMain, EffectPriority, UpdateEffects};

pub mod effects;

pub struct Value<T>
where
    T: Copy + Send + Sync + AddAssign + MulAssign,
{
    value: T,
    base: T,
    rx: mpsc::Receiver<UpdateEffects>,
    tx: Arc<mpsc::Sender<UpdateEffects>>,
    effects: Vec<EffectMain<T>>,
}

impl<T> Value<T>
where
    T: Copy + Send + Sync + AddAssign + MulAssign,
{
    pub fn new(base: T) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            base,
            value: base,
            rx,
            tx: Arc::new(tx),
            effects: Vec::new(),
        }
    }

    pub fn get(&mut self) -> T {
        let mut rx_iter = self.rx.try_iter().peekable();
        if rx_iter.peek().is_some() {
            self.effects
                .retain(|effect| effect.action.upgrade().is_some());
            self.value = self.base;
            self.recalculate();
        }
        self.value
    }

    fn find_copy(&self, effect: &EffectMain<T>) -> Option<usize> {
        self.effects.iter().position(|other_effect| {
            if effect.id == other_effect.id {
                true
            } else {
                false
            }
        })
    }

    fn recalculate(&mut self) {
        let mut groups: Vec<EffectPriority> = Vec::new();
        for effect in self.effects.iter() {
            if let Some(group) = groups
                .iter()
                .find(|group| effect.priority.group == group.group)
            {
                if effect.priority.value != group.value {
                    return;
                }
                if let Some(effect_ref) = effect.action.upgrade() {
                    effect_ref.apply(&mut self.value);
                }
            } else if let Some(effect_ref) = effect.action.upgrade() {
                groups.push(effect.priority);
                effect_ref.apply(&mut self.value);
            }
        }
    }
}
