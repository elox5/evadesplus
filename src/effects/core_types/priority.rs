use super::PriorityGroup;
use crate::effects::group_table;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct EffectPriority {
    pub group: PriorityGroup,
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
        self.group != other.group || self.value == other.value
    }
}

impl Eq for EffectPriority {}

impl PartialOrd for EffectPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
