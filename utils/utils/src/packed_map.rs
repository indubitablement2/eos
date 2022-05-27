use crate::*;
use ahash::AHashMap;
use std::{hash::Hash, marker::PhantomData};

/// A map where element are contiguous in memory.
///
/// Similar to IndexMap, but generic over the underlying container.
pub struct PackedMap<C: Container<T>, T: soak::Columns, I: Hash + Eq + Copy> {
    container: C,
    id_vec: Vec<I>,
    index_map: AHashMap<I, usize>,
    _marker: PhantomData<T>,
}
impl<C: Container<T>, T: soak::Columns, I: Hash + Eq + Copy> PackedMap<C, T, I> {
    /// Constructs a new, empty container with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            container: C::with_capacity(capacity),
            id_vec: Vec::with_capacity(capacity),
            index_map: AHashMap::with_capacity(capacity),
            _marker: Default::default(),
        }
    }

    // /// Appends an element to the back of the container.
    // pub fn push(&mut self, value: T) -> (I, usize) {
    //     let id = self.next_id;
    //     self.next_id.increment();

    //     let index = self.len();

    //     self.container.push(value);
    //     self.id_vec.push(id);
    //     self.index_map.insert(id, index);

    //     (id, index)
    // }

    /// Insert a value with a predefined id.
    ///
    /// Return the index it was insert at and the old value (if there was some).
    pub fn insert(&mut self, id: I, value: T) -> (usize, Option<T>) {
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
    pub fn pop(&mut self) -> Option<(T, I, usize)> {
        if let Some(id) = self.id_vec.pop() {
            let index = self.index_map.remove(&id).unwrap();
            let value = self.container.pop().unwrap();
            Some((value, id, index))
        } else {
            None
        }
    }

    /// Removes an element from the container and returns it.
    ///
    /// The removed element is replaced by the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    pub fn swap_remove_by_index(&mut self, index: usize) -> Option<(T, I)> {
        if index >= self.len() {
            None
        } else {
            let id = self.id_vec.swap_remove(index);
            self.index_map.remove(&id).unwrap();
            let value = self.container.swap_remove(index);

            if let Some(moved_id) = self.id_vec.get(index) {
                *self.index_map.get_mut(moved_id).unwrap() = index;
            }

            Some((value, id))
        }
    }

    /// Removes an element from the container and returns it.
    ///
    /// The removed element is replaced by the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    pub fn swap_remove_by_id(&mut self, id: I) -> Option<(T, usize)> {
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
    }

    pub fn container(&mut self) -> &mut C {
        &mut self.container
    }

    pub fn id_vec(&self) -> &[I] {
        self.id_vec.as_ref()
    }
}

impl<T: soak::Columns + components::Components, I: Hash + Eq + Copy> PackedMap<Soa<T>, T, I> {
    /// Shortcut for `this.container().raw_table()`.
    /// Allow using PackedMap<Soa<T>, T, I> in query!() directly.
    pub fn raw_table(&mut self) -> &mut RawTable<T> {
        &mut self.container.raw_table
    }
}
