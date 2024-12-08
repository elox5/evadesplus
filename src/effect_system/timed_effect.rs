use std::{
    sync::{mpsc, Arc, OnceLock, Weak},
    time::Duration,
};

use tokio::task::JoinHandle;

use crate::effect_system::{
    effect::{Effect, EffectAction, EffectMain, UpdateEffects},
    priority::EffectPriority,
    target::{EffectId, EffectTarget},
};

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
    pub fn apply(
        id: EffectId,
        priority: EffectPriority,
        effect: EffectAction<T::EffectValue>,
        target_list: &mut Vec<&mut T>,
        duration: Duration,
    ) -> Weak<Self> {
        let effect = Arc::new(effect);
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

impl<T> Effect for Weak<TimedEffect<T>>
where
    T: EffectTarget,
{
    fn clear(self) {
        if let Some(effect) = self.upgrade() {
            unsafe { effect.handle.get().unwrap_unchecked() }.abort();
        }
    }
}
