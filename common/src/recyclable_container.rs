

#[derive(Clone, Default)]
pub struct Components<T> {
    data: Vec<T>,
    free_index: Vec<usize>,
}
impl<T> Components<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            free_index: (0..capacity).collect(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    pub fn push(&mut self, t: T) -> usize {
        if let Some(index) = self.free_index.pop() {
            unsafe{*self.data.get_unchecked_mut(index) = t;}
            index
        } else {
            let index = self.data.len();
            self.data.push(t);
            index
        }
    }

    /// # Safety
    /// Ensure that nothing is referencing this index as it will be reused.
    pub unsafe fn remove(&mut self, index: usize) {
        if self.data.len() > index {
            self.free_index.push(index)
        }
    }
}