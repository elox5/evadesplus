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
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync + Mul<f32, Output = TAdd>,
    TMul: Copy + Send + Sync + Mul<f32, Output = TMul>,
{
    value: T,
    base: T,
    base_effect: EffectAction<T, TAdd, TMul>,
    effect_receptivity: f32,
    rx: mpsc::Receiver<UpdateEffects>,
    tx: mpsc::Sender<UpdateEffects>,
    effects: Vec<EffectMain<T, TAdd, TMul>>,
}

impl<T, TAdd, TMul> Value<T, TAdd, TMul>
where
    T: Clone + Copy + Send + Sync + Add<TAdd, Output = T> + Mul<TMul, Output = T>,
    TAdd: Copy + Send + Sync + Mul<f32, Output = TAdd>,
    TMul: Copy + Send + Sync + Mul<f32, Output = TMul>,
{
    pub fn new(base: T, effect_receptivity: f32) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            base,
            value: base,
            base_effect: EffectAction::None,
            effect_receptivity,
            rx,
            tx,
            effects: Vec::new(),
        }
    }

    pub fn update_base(&mut self, action: EffectAction<T, TAdd, TMul>) {
        self.base_effect = action;
    }

    pub fn set_base(&mut self, value: T) {
        self.base = value;
    }

    pub fn get_base(&self) -> T {
        self.base
    }

    pub fn set_receptivity(&mut self, effect_receptivity: f32) {
        if effect_receptivity != self.effect_receptivity {
            self.rx.try_iter();
            self.effect_receptivity = effect_receptivity;
            self.effects
                .retain(|effect| effect.action.upgrade().is_some());
            self.recalculate();
        }
    }

    pub fn get(&mut self) -> T {
        if self.rx.try_iter().next().is_some() {
            self.effects
                .retain(|effect| effect.action.upgrade().is_some());
            self.recalculate();
        }
        self.value
    }

    fn recalculate(&mut self) {
        self.value = self.base;
        self.base_effect
            .apply_without_receptivity_to(&mut self.value);
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
                    let effect_ref = effect_ref.load();
                    if effect_ref.0 {
                        effect_ref
                            .1
                            .apply_to(&mut self.value, self.effect_receptivity);
                    } else {
                        effect_ref.1.apply_without_receptivity_to(&mut self.value);
                    }
                    drop(effect_ref);
                    ids.push(effect.id);
                }
            } else if let Some(effect_ref) = effect.action.upgrade() {
                let effect_ref = effect_ref.load();
                if effect_ref.0 {
                    effect_ref
                        .1
                        .apply_to(&mut self.value, self.effect_receptivity);
                } else {
                    effect_ref.1.apply_without_receptivity_to(&mut self.value);
                }
                drop(effect_ref);
                groups.push(effect.priority);
                ids.push(effect.id);
            }
        }
    }
}
