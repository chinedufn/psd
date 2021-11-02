use crate::PsdGroup;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug)]
pub(crate) struct Groups {
    groups: HashMap<u32, PsdGroup>,
    group_ids_in_order: Vec<u32>,
}

impl Groups {
    pub fn with_capacity(capacity: usize) -> Self {
        Groups {
            groups: HashMap::with_capacity(capacity),
            group_ids_in_order: Vec::with_capacity(capacity),
        }
    }

    /// Add a group to the list of groups, making it last in the order.
    pub fn push(&mut self, group: PsdGroup) {
        self.group_ids_in_order.push(group.id);

        self.groups.insert(group.id, group);
    }

    /// Get the group ID's in order (from bottom to top in a PSD file).
    pub fn group_ids_in_order(&self) -> &Vec<u32> {
        &self.group_ids_in_order
    }
}

impl Deref for Groups {
    type Target = HashMap<u32, PsdGroup>;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}
