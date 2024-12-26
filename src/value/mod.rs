use crate::effects::core_types::{EffectId, EffectMain, EffectPriority, UpdateEffects};
use std::{
    ops::{AddAssign, MulAssign},
    sync::{mpsc, Arc},
};

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
        let mut rx_iter = self.rx.try_iter();
        if rx_iter.next().is_some() {
            self.effects
                .retain(|effect| effect.action.upgrade().is_some());
            self.recalculate();
        }
        self.value
    }

    fn recalculate(&mut self) {
        self.value = self.base;
        let mut groups: Vec<EffectPriority> = Vec::new();
        let mut ids: Vec<EffectId> = Vec::new();
        for effect in self.effects.iter() {
            if let Some(group) = groups
                .iter()
                .find(|group| effect.priority.group == group.group)
            {
                if effect.priority.value != group.value || ids.contains(&effect.id) {
                    continue;
                }
                if let Some(effect_ref) = effect.action.upgrade() {
                    effect_ref.apply_to(&mut self.value);
                    ids.push(effect.id);
                }
            } else if let Some(effect_ref) = effect.action.upgrade() {
                effect_ref.apply_to(&mut self.value);
                groups.push(effect.priority);
                ids.push(effect.id);
            }
        }
    }
}
