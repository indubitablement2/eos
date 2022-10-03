use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Some common functions for container.
pub trait Container: Sized {
    type Item;
    /// Constructs a new, empty container with the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
    /// Constructs a new, empty container without allocating.
    fn new() -> Self {
        Self::with_capacity(0)
    }
    /// Appends an element to the back of the container.
    fn push(&mut self, value: Self::Item);
    /// Removes the last element and returns it, or None if it is empty.
    fn pop(&mut self) -> Option<Self::Item>;
    /// Removes an element from the container and returns it.
    ///
    /// The removed element is replaced by the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    fn swap_remove(&mut self, index: usize) -> Self::Item;
    /// Replace element at index with a value returning the old value.
    fn replace(&mut self, index: usize, value: Self::Item) -> Self::Item;
    /// Returns the number of elements in the container.
    fn len(&self) -> usize;
    /// Returns the number of elements the container can hold without reallocating.
    fn capacity(&self) -> usize;
    /// Reserves capacity for at least additional more elements to be inserted.
    fn reserve(&mut self, additional: usize);
    /// Swaps two elements.
    fn swap_elements(&mut self, a: usize, b: usize);
}
impl<T> Container for Vec<T> {
    type Item = T;

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn push(&mut self, value: Self::Item) {
        self.push(value)
    }

    fn pop(&mut self) -> Option<Self::Item> {
        self.pop()
    }

    fn swap_remove(&mut self, index: usize) -> Self::Item {
        self.swap_remove(index)
    }

    fn replace(&mut self, index: usize, mut value: Self::Item) -> Self::Item {
        std::mem::swap(self.get_mut(index).unwrap(), &mut value);
        value
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }

    fn swap_elements(&mut self, a: usize, b: usize) {
        self.swap(a, b)
    }
}

/// A map where element are contiguous in memory.
///
/// Similar to IndexMap, but generic over the underlying container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedMap<C, I>
where
    C: Container,
    I: Hash + Eq + Copy,
{
    /// Elements order (push, pop, swap, etc.) should not be changed manualy.
    pub container: C,
    /// The idx of the elements at the same index.
    pub id_vec: Vec<I>,
    /// The index of the idx.
    pub index_map: AHashMap<I, usize>,
}
impl<C, I> PackedMap<C, I>
where
    C: Container,
    I: Hash + Eq + Copy,
{
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Constructs a new, empty container with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            container: C::with_capacity(capacity),
            id_vec: Vec::with_capacity(capacity),
            index_map: AHashMap::with_capacity(capacity),
        }
    }

    /// Insert a value with a predefined id.
    ///
    /// Return the index it was inserted at and the old value (if there was some).
    pub fn insert(&mut self, id: I, value: C::Item) -> (usize, Option<C::Item>) {
        if let Some(&index) = self.index_map.get(&id) {
            (index, Some(self.container.replace(index, value)))
        } else {
            let index = self.len();
            self.id_vec.push(id);
            self.index_map.insert(id, index);
            self.container.push(value);
            (index, None)
        }
    }

    /// Removes the last element and returns it, or None if it is empty.
    pub fn pop(&mut self) -> Option<(C::Item, I, usize)> {
        if let Some(id) = self.id_vec.pop() {
            let index = self.index_map.remove(&id).unwrap();
            let value = self.container.pop().unwrap();
            Some((value, id, index))
        } else {
            None
        }
    }

    /// ## Panic:
    /// If indices are out of bound.
    pub fn swap_by_index(&mut self, a: usize, b: usize) {
        self.container.swap_elements(a, b);
        *self.index_map.get_mut(&self.id_vec[a]).unwrap() = b;
        *self.index_map.get_mut(&self.id_vec[b]).unwrap() = a;
        self.id_vec.swap(a, b);
    }

    /// Removes an element from the container and returns it.
    ///
    /// The removed element is replaced by the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// ## Panic:
    /// Panic if index is out of bound.
    pub fn swap_remove_by_index(&mut self, index: usize) -> (C::Item, I) {
        let id = self.id_vec.swap_remove(index);
        self.index_map.remove(&id).unwrap();
        let value = self.container.swap_remove(index);

        if let Some(moved_id) = self.id_vec.get(index) {
            *self.index_map.get_mut(moved_id).unwrap() = index;
        }

        (value, id)
    }

    /// Removes an element from the container and returns it.
    ///
    /// The removed element is replaced by the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    pub fn swap_remove_by_id(&mut self, id: I) -> Option<(C::Item, usize)> {
        if let Some(index) = self.index_map.remove(&id) {
            self.id_vec.swap_remove(index);
            let value = self.container.swap_remove(index);

            if let Some(moved_id) = self.id_vec.get(index) {
                *self.index_map.get_mut(moved_id).unwrap() = index;
            }

            Some((value, index))
        } else {
            None
        }
    }

    /// Return the index of the value with the provided id.
    pub fn get_index(&self, id: I) -> Option<usize> {
        self.index_map.get(&id).copied()
    }

    /// Return the id of the value at index.
    pub fn get_id(&self, index: usize) -> Option<I> {
        self.id_vec.get(index).copied()
    }

    /// Returns the number of elements in the container.
    pub fn len(&self) -> usize {
        self.id_vec.len()
    }

    /// Returns the number of elements the container can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.container.capacity()
    }

    /// Reserves capacity for at least additional more elements to be inserted.
    pub fn reserve(&mut self, additional: usize) {
        self.container.reserve(additional);
        self.id_vec.reserve(additional);
        self.index_map.reserve(additional);
    }

    pub fn id_vec(&self) -> &[I] {
        self.id_vec.as_ref()
    }
}

impl<C, I> Extend<(I, C::Item)> for PackedMap<C, I>
where
    C: Container,
    I: Hash + Eq + Copy,
{
    fn extend<T: IntoIterator<Item = (I, C::Item)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        self.reserve(hint.1.unwrap_or(hint.0));
        for (id, value) in iter {
            self.insert(id, value);
        }
    }
}

impl<C, I> FromIterator<(I, C::Item)> for PackedMap<C, I>
where
    C: Container,
    I: Hash + Eq + Copy,
{
    fn from_iter<T: IntoIterator<Item = (I, C::Item)>>(iter: T) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}
