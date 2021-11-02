use crate::PsdLayer;
use std::collections::HashMap;
use std::ops::{Deref, Range};

/// `NamedItems` is immutable container for storing items with order-preservation
/// and indexing by id and name
#[derive(Debug)]
pub(crate) struct Layers {
    items: Vec<PsdLayer>,
    // TODO: This is incorrect since layers can have the same name. Perhaps a Vec<u32> instead of
    //  usize. Add a failing test with a PSD with two layers with the same name.
    //  Take inspiration from the `Groups` type.
    item_indices: HashMap<String, usize>,
}

impl Layers {
    /// Creates a new `NamedItems`
    pub fn new() -> Self {
        Layers {
            items: vec![],
            item_indices: HashMap::new(),
        }
    }

    /// Creates a new `NamedItems` with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Layers {
            items: Vec::with_capacity(capacity),
            item_indices: HashMap::with_capacity(capacity),
        }
    }

    #[allow(missing_docs)]
    pub fn item_by_name(&self, name: &str) -> Option<&PsdLayer> {
        match self.item_indices.get(name) {
            Some(item_idx) => self.items.get(*item_idx),
            None => None,
        }
    }

    #[allow(missing_docs)]
    pub(in crate) fn push(&mut self, name: String, item: PsdLayer) {
        self.items.push(item);
        self.item_indices.insert(name, self.items.len() - 1);
    }
}

impl Deref for Layers {
    type Target = Vec<PsdLayer>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
