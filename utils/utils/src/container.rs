/// Some common functions for container.
pub trait Container<T> {
    /// Constructs a new, empty container with the specified capacity.
    fn with_capacity(capacity: usize) -> Self;
    /// Appends an element to the back of the container.
    fn push(&mut self, value: T);
    /// Removes the last element and returns it, or None if it is empty.
    fn pop(&mut self) -> Option<T>;
    /// Removes an element from the container and returns it.
    ///
    /// The removed element is replaced by the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    fn swap_remove(&mut self, index: usize) -> T;
    /// Replace element at index with a value returning the old value.
    fn replace(&mut self, index: usize, value: T) -> T;
    /// Returns the number of elements in the container.
    fn len(&self) -> usize;
    /// Returns the number of elements the container can hold without reallocating.
    fn capacity(&self) -> usize;
    /// Reserves capacity for at least additional more elements to be inserted.
    fn reserve(&mut self, additional: usize);
}
impl<T: Extend<T> + std::marker::Copy> Container<T> for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    fn push(&mut self, value: T) {
        self.push(value)
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn swap_remove(&mut self, index: usize) -> T {
        self.swap_remove(index)
    }

    fn replace(&mut self, index: usize, mut value: T) -> T {
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
}
