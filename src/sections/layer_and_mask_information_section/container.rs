use std::collections::HashMap;
use std::ops::{Index, Range};

#[derive(Debug)]
pub(crate) struct KeyIDContainer<T> {
    items: Vec<T>,
    item_indices: HashMap<String, usize>,
}

impl<T> KeyIDContainer<T> {
    /// Creates a new `KeyIDContainer`
    pub fn new() -> Self {
        KeyIDContainer {
            items: vec![],
            item_indices: HashMap::new(),
        }
    }

    /// Creates a new `KeyIDContainer` with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        KeyIDContainer {
            items: Vec::with_capacity(capacity),
            item_indices: HashMap::with_capacity(capacity),
        }
    }

    /// Get all of the items in the container
    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    /// Get a item by name
    pub fn item_by_name(&self, name: &str) -> Option<&T> {
        match self.item_indices.get(name) {
            Some(item_idx) => self.items.get(*item_idx),
            None => None
        }
    }

    /// Get a item by index
    pub fn item_by_idx(&self, idx: usize) -> Option<&T> {
        self.items.get(idx)
    }

    /// Get a range of items
    pub fn range(&self, start: usize, end: usize) -> &[T] {
        &self.items[start..end]
    }

    /// Returns number of items in container
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Adds new item to container
    pub fn push(&mut self, name: String, item: T) {
        self.items.push(item);
        self.item_indices.insert(name, self.items.len() - 1);
    }
}

impl<T> Index<usize> for KeyIDContainer<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.item_by_idx(idx).unwrap()
    }
}

impl<T> Index<&str> for KeyIDContainer<T> {
    type Output = T;

    fn index(&self, name: &str) -> &Self::Output {
        &self.item_by_name(name).unwrap()
    }
}

impl<T> Index<&Range<usize>> for KeyIDContainer<T> {
    type Output = [T];

    fn index(&self, range: &Range<usize>) -> &Self::Output {
        self.range(range.start, range.end)
    }
}