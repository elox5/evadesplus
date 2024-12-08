use std::{collections::HashMap, sync::LazyLock};

use arc_swap::ArcSwap;

pub static GROUP_TABLE: LazyLock<ArcSwap<HashMap<&str, u16>>> =
    LazyLock::new(|| ArcSwap::new(HashMap::from([("group", 0), ("other_group", 1)]).into()));

pub fn get(group: &'static str) -> u16 {
    *GROUP_TABLE.load().get(group).expect(&format!(
        "Effect group \"{group}\" not found in initial group table"
    ))
}

pub fn get_or_insert(name: String) -> u16 {
    let table = GROUP_TABLE.load();
    if let Some(id) = table.get(&&name[..]) {
        *id
    } else {
        let leak = Box::leak(name.into_boxed_str());
        let mut table = (**table).clone();
        let len = table.len() as u16;
        table.insert(leak, len);
        GROUP_TABLE.store(table.into());
        len
    }
}
