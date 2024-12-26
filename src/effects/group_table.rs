use super::core_types::PriorityGroup;
use arc_swap::ArcSwap;
use std::{collections::HashMap, sync::LazyLock};

pub static GROUP_TABLE: LazyLock<ArcSwap<HashMap<&str, PriorityGroup>>> =
    LazyLock::new(|| ArcSwap::new(HashMap::from([("group", 0), ("other_group", 1)]).into()));

pub fn get(group: &'static str) -> PriorityGroup {
    *GROUP_TABLE
        .load()
        .get(group)
        .unwrap_or_else(|| panic!("Effect group \"{group}\" not found in initial group table"))
}

pub fn get_or_insert(name: String) -> PriorityGroup {
    let table = GROUP_TABLE.load();
    if let Some(id) = table.get(&&name[..]) {
        *id
    } else {
        let leak = Box::leak(name.into_boxed_str());
        let mut table = (**table).clone();
        let len = table.len() as PriorityGroup;
        table.insert(leak, len);
        GROUP_TABLE.store(table.into());
        len
    }
}
