use super::{
    core_types::{EffectAction, EffectId, EffectMain, EffectPriority, EffectStore, UpdateEffects},
    target::EffectTarget,
};
use arc_swap::ArcSwap;
use std::{
    sync::{mpsc, Arc, OnceLock, Weak},
    time::Duration,
};
use tokio::task::JoinHandle;

pub struct TimedEffect<T>
where
    T: EffectTarget,
{
    effect: EffectStore<T>,
    targets: Vec<mpsc::Sender<UpdateEffects>>,
    handle: OnceLock<JoinHandle<()>>,
}

impl<T> TimedEffect<T>
where
    T: EffectTarget + 'static,
{
    pub(super) fn apply(
        id: EffectId,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>,
        target_list: &mut Vec<&mut T>,
        duration: Duration,
    ) -> Weak<Self> {
        let effect = Arc::new(ArcSwap::new(Arc::new(action)));
        let new = Self {
            targets: target_list
                .iter_mut()
                .map(|target| target.apply(EffectMain::new(id, priority, &effect)))
                .collect(),
            effect,
            handle: OnceLock::new(),
        };
        let new_arc = Arc::new(new);
        let handle_arc = new_arc.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            drop(handle_arc);
        });
        let _ = new_arc.handle.set(handle);
        Arc::downgrade(&new_arc)
    }

    pub(super) fn clear(&self) {
        unsafe { self.handle.get().unwrap_unchecked() }.abort();
    }

    pub(super) fn get(&self) -> EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul> {
        **self.effect.load()
    }

    pub(super) fn update(&self, action: EffectAction<T::EffectValue, T::EffectAdd, T::EffectMul>) {
        self.effect.store(Arc::new(action));
    }
}

impl<T> Drop for TimedEffect<T>
where
    T: EffectTarget,
{
    fn drop(&mut self) {
        self.targets.iter().for_each(|target| {
            let _ = target.send(UpdateEffects);
        });
    }
}
