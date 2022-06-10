use crate::*;

#[derive(Default)]
pub struct Soa<T: soak::Columns + Components> {
    pub raw_table: RawTable<T>,
    len: usize,
}
impl<T: _components::Components> Drop for Soa<T> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                T::move_from_table(&mut self.raw_table, i);
            }
        }
    }
}
impl<T: soak::Columns + Components> Soa<T> {
    pub fn raw_table(&mut self) -> &mut RawTable<T> {
        &mut self.raw_table
    }
}
impl<T: Components> Extend<T> for Soa<T>
where
    Soa<T>: Container<T>,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();

        // Reserve space for the iterator.
        let hint = iter.size_hint();
        let needed = hint.1.unwrap_or(hint.0);
        let available = self.capacity() - self.len();
        if available < needed {
            self.reserve(needed - available);
        }

        for value in iter {
            self.push(value)
        }
    }
}
impl<T> Container<T> for Soa<T>
where
    T: Components,
{
    fn with_capacity(capacity: usize) -> Self {
        Self {
            raw_table: RawTable::with_capacity(capacity),
            len: 0,
        }
    }

    fn push(&mut self, value: T) {
        if self.len >= self.capacity() {
            self.reserve(1);
        }

        unsafe {
            value.move_to_table(&mut self.raw_table, self.len);
        }

        self.len += 1;
    }

    fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(T::move_from_table(&mut self.raw_table, self.len)) }
        }
    }

    fn swap_remove(&mut self, index: usize) -> T {
        self.len -= 1;

        unsafe {
            // Take value at index.
            let removed = T::move_from_table(&mut self.raw_table, index);

            // Move last to index.
            if index != self.len {
                let to_move = T::move_from_table(&mut self.raw_table, self.len);
                to_move.move_to_table(&mut self.raw_table, index);
            }

            removed
        }
    }

    fn replace(&mut self, index: usize, value: T) -> T {
        unsafe {
            // Drop value at index.
            let removed = T::move_from_table(&mut self.raw_table, index);

            // Copy value to index.
            value.move_to_table(&mut self.raw_table, index);

            removed
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.raw_table.capacity()
    }

    fn reserve(&mut self, mut additional: usize) {
        if additional == 0 {
            return;
        }

        // Never allocate less than previous capacity.
        additional = additional.max(self.capacity());

        self.raw_table
            .reserve_exact(self.raw_table.capacity(), additional)
    }

    fn swap_elements(&mut self, a: usize, b: usize) {
        todo!()
    }
}
