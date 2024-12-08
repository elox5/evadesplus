use std::sync::{mpsc, Arc, Weak};

use crate::effect_system::{
    effect::{Effect, EffectAction, EffectMain, UpdateEffects},
    priority::EffectPriority,
    target::{EffectId, EffectTarget},
};

pub struct TogglableEffect<T>
where
    T: EffectTarget,
{
    _effect: Arc<EffectAction<T::EffectValue>>,
    targets: Vec<Weak<mpsc::Sender<UpdateEffects>>>,
}

impl<T> TogglableEffect<T>
where
    T: EffectTarget + 'static,
{
    pub fn apply(
        id: EffectId,
        priority: EffectPriority,
        effect: EffectAction<T::EffectValue>,
        target_list: &mut Vec<&mut T>,
    ) -> Self {
        let effect = Arc::new(effect);
        let new = Self {
            targets: target_list
                .iter_mut()
                .map(|target| target.apply(EffectMain::new(id, priority, &effect)))
                .collect(),
            _effect: effect,
        };
        new
    }
}

impl<T> Drop for TogglableEffect<T>
where
    T: EffectTarget,
{
    fn drop(&mut self) {
        self.targets.iter().for_each(|target| {
            if let Some(target) = target.upgrade() {
                let _ = target.send(UpdateEffects);
            }
        });
    }
}

impl<T> Effect for TogglableEffect<T>
where
    T: EffectTarget + 'static,
{
    fn clear(self) {}
}
