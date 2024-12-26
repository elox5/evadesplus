use crate::effects::core_types::{
    EffectAction, EffectId, EffectMain, EffectPriority, UpdateEffects,
};
use std::{
    ops::{Add, Mul},
    sync::mpsc,
};

pub mod effects;

pub struct Value<T, TAdd = T, TMul = T>
where
    T: Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync,
    TMul: Copy + Send + Sync,
{
    value: T,
    base: T,
    base_effect: EffectAction<T, TAdd, TMul>,
    rx: mpsc::Receiver<UpdateEffects>,
    tx: mpsc::Sender<UpdateEffects>,
    effects: Vec<EffectMain<T, TAdd, TMul>>,
}

impl<T, TAdd, TMul> Value<T, TAdd, TMul>
where
    T: Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync,
    TMul: Copy + Send + Sync,
{
    pub fn new(base: T) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            base,
            value: base,
            base_effect: EffectAction::None,
            rx,
            tx,
            effects: Vec::new(),
        }
    }

    pub fn update_base(&mut self, action: EffectAction<T, TAdd, TMul>) {
        self.base_effect = action;
    }

    pub fn get_base(&self) -> T {
        self.base
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
        self.base_effect.apply_to(&mut self.value);
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
                    effect_ref.load().apply_to(&mut self.value);
                    ids.push(effect.id);
                }
            } else if let Some(effect_ref) = effect.action.upgrade() {
                effect_ref.load().apply_to(&mut self.value);
                groups.push(effect.priority);
                ids.push(effect.id);
            }
        }
    }
}
