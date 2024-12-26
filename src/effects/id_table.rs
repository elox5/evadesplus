use super::core_types::EffectId;
use arc_swap::ArcSwap;
use std::{collections::HashMap, sync::LazyLock};

pub static ID_TABLE: LazyLock<ArcSwap<HashMap<&str, EffectId>>> =
    LazyLock::new(|| ArcSwap::new(HashMap::from([]).into()));

pub fn get(id: &'static str) -> EffectId {
    *ID_TABLE
        .load()
        .get(id)
        .expect(&format!("Effect id \"{id}\" not found in initial id table"))
}

pub fn get_or_insert(name: String) -> EffectId {
    let table = ID_TABLE.load();
    if let Some(id) = table.get(&&name[..]) {
        *id
    } else {
        let leak = Box::leak(name.into_boxed_str());
        let mut table = (**table).clone();
        let len = table.len() as EffectId;
        table.insert(leak, len);
        ID_TABLE.store(table.into());
        len
    }
}
