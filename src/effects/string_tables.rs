use std::{collections::HashMap, sync::LazyLock};

use arc_swap::ArcSwap;

use super::core_types::{EffectId, PriorityGroup};

pub static GROUP_TABLE: StaticStringTable<PriorityGroup> = StaticStringTable::new(
    || ArcSwap::new(HashMap::from([]).into()),
    |group| {
        #[cfg(debug_assertions)]
        panic!("Effect group '{group}' not found in initial group table");
        #[cfg(not(debug_assertions))]
        hint::unreachable_unchecked();
    },
);
pub static ID_TABLE: StaticStringTable<EffectId> = StaticStringTable::new(
    || ArcSwap::new(HashMap::from([]).into()),
    |id| {
        #[cfg(debug_assertions)]
        panic!("Effect id '{id}' not found in initial id table");
        #[cfg(not(debug_assertions))]
        hint::unreachable_unchecked();
    },
);

pub struct StaticStringTable<T>(
    LazyLock<ArcSwap<HashMap<&'static str, T>>>,
    fn(&'static str) -> &'static T,
)
where
    T: Clone + Copy + TryFrom<usize> + 'static;

impl<T> StaticStringTable<T>
where
    T: Clone + Copy + TryFrom<usize>,
{
    pub const fn new(
        from: fn() -> ArcSwap<HashMap<&'static str, T>>,
        on_get_fail: fn(&'static str) -> &'static T,
    ) -> Self {
        Self(LazyLock::new(from), on_get_fail)
    }

    pub fn get(&self, group: &'static str) -> T {
        *self.0.load().get(group).unwrap_or_else(|| self.1(group))
    }

    pub fn get_or_insert(&self, name: String) -> T {
        let table = self.0.load();
        if let Some(id) = table.get(&&name[..]) {
            *id
        } else {
            let leak = Box::leak(name.into_boxed_str());
            let mut table = (**table).clone();
            let len = unsafe { table.len().try_into().unwrap_unchecked() };
            table.insert(leak, len);
            self.0.store(table.into());
            len
        }
    }
}
