use std::cmp::Ordering;

use crate::effect_system::group_table;

#[derive(Clone, Copy)]
pub struct EffectPriority {
    pub group: u16,
    pub value: u8,
}

impl EffectPriority {
    pub fn new(group: &'static str, value: u8) -> Self {
        Self {
            group: group_table::get(group),
            value,
        }
    }

    pub fn from_string(group: String, value: u8) -> Self {
        Self {
            group: group_table::get_or_insert(group),
            value,
        }
    }
}

impl PartialEq for EffectPriority {
    fn eq(&self, other: &Self) -> bool {
        self.group != other.group || (self.group == other.group && self.value == other.value)
    }
}

impl Eq for EffectPriority {}

impl PartialOrd for EffectPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.group == other.group {
            self.value.partial_cmp(&other.value)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for EffectPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.group == other.group {
            self.value.cmp(&other.value)
        } else {
            Ordering::Equal
        }
    }
}
