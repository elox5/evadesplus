use super::{
    core_types::{EffectAction, EffectId, EffectMain, EffectPriority, UpdateEffects},
    target::EffectTarget,
};
use std::{
    sync::{mpsc, Arc, OnceLock, Weak},
    time::Duration,
};
use tokio::task::JoinHandle;

pub struct TimedEffect<T>
where
    T: EffectTarget,
{
    _effect: Arc<EffectAction<T::EffectValue>>,
    targets: Vec<Weak<mpsc::Sender<UpdateEffects>>>,
    handle: OnceLock<JoinHandle<()>>,
}

impl<T> TimedEffect<T>
where
    T: EffectTarget + 'static,
{
    pub(super) fn apply(
        id: EffectId,
        priority: EffectPriority,
        action: EffectAction<T::EffectValue>,
        target_list: &mut Vec<&mut T>,
        duration: Duration,
    ) -> Weak<Self> {
        let effect = Arc::new(action);
        let new = Self {
            targets: target_list
                .iter_mut()
                .map(|target| target.apply(EffectMain::new(id, priority, &effect)))
                .collect(),
            _effect: effect,
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
}

impl<T> Drop for TimedEffect<T>
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
