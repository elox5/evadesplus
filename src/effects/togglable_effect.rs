use arc_swap::ArcSwap;

use super::{
    core_types::{EffectAction, EffectId, EffectMain, EffectPriority, EffectStore, UpdateEffects},
    target::EffectTarget,
};
use std::sync::{mpsc, Arc};

pub struct TogglableEffect<T>
where
    T: EffectTarget,
{
    effect: EffectStore<T>,
    targets: Vec<mpsc::Sender<UpdateEffects>>,
}

impl<T> TogglableEffect<T>
where
    T: EffectTarget + 'static,
{
    pub(super) fn apply(
        target_list: &mut Vec<&mut T>,
        id: EffectId,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ignore_receptivity: bool,
    ) -> Self {
        let effect = Arc::new(ArcSwap::new(Arc::new((ignore_receptivity, action))));
        let new = Self {
            targets: target_list
                .iter_mut()
                .map(|target| target.apply(EffectMain::new(id, priority, &effect)))
                .collect(),
            effect,
        };
        new
    }

    pub(super) fn get(
        &self,
    ) -> (
        bool,
        EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
    ) {
        **self.effect.load()
    }

    pub(super) fn update(
        &self,
        action: (
            bool,
            EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        ),
    ) {
        self.effect.store(Arc::new(action));
    }
}

impl<T> Drop for TogglableEffect<T>
where
    T: EffectTarget,
{
    fn drop(&mut self) {
        self.targets.iter().for_each(|target| {
            let _ = target.send(UpdateEffects);
        });
    }
}
