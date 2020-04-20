use std::collections::HashMap;
use std::ops::{Index, Range};

/// `NamedItems` is immutable container for storing items with order-preservation
/// and indexing by id and name
#[derive(Debug)]
pub(crate) struct NamedItems<T> {
    items: Vec<T>,
    item_indices: HashMap<String, usize>,
}

impl<T> NamedItems<T> {
    /// Creates a new `NamedItems`
    pub fn new() -> Self {
        NamedItems {
            items: vec![],
            item_indices: HashMap::new(),
        }
    }

    /// Creates a new `NamedItems` with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        NamedItems {
            items: Vec::with_capacity(capacity),
            item_indices: HashMap::with_capacity(capacity),
        }
    }

    #[allow(missing_docs)]
    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    #[allow(missing_docs)]
    pub fn item_by_name(&self, name: &str) -> Option<&T> {
        match self.item_indices.get(name) {
            Some(item_idx) => self.items.get(*item_idx),
            None => None,
        }
    }

    #[allow(missing_docs)]
    pub fn item_by_idx(&self, idx: usize) -> Option<&T> {
        self.items.get(idx)
    }

    #[allow(missing_docs)]
    pub fn range(&self, start: usize, end: usize) -> &[T] {
        &self.items[start..end]
    }

    #[allow(missing_docs)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[allow(missing_docs)]
    pub(in crate) fn push(&mut self, name: String, item: T) {
        self.items.push(item);
        self.item_indices.insert(name, self.items.len() - 1);
    }
}

impl<T> Index<usize> for NamedItems<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.item_by_idx(idx).unwrap()
    }
}

impl<T> Index<&str> for NamedItems<T> {
    type Output = T;

    fn index(&self, name: &str) -> &Self::Output {
        &self.item_by_name(name).unwrap()
    }
}

impl<T> Index<&Range<usize>> for NamedItems<T> {
    type Output = [T];

    fn index(&self, range: &Range<usize>) -> &Self::Output {
        self.range(range.start, range.end)
    }
}
