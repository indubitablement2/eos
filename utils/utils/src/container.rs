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
