use super::PriorityGroup;
use crate::effects::string_tables::GROUP_TABLE;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct EffectPriority {
    pub group: PriorityGroup,
    pub value: u8,
}

impl EffectPriority {
    pub fn new(group: &'static str, value: u8) -> Self {
        Self {
            group: GROUP_TABLE.get(group),
            value,
        }
    }

    pub fn from_string(group: String, value: u8) -> Self {
        Self {
            group: GROUP_TABLE.get_or_insert(group),
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
