pub struct Test {
    a: u32,
    b: Vec<u32>,
    last: std::boxed::Box<u32>,
}
pub struct TestSoa {
    pub a: Vec<u32>,
    pub b: Vec<Vec<u32>>,
    pub last: Vec<std::boxed::Box<u32>>,
}
impl ::utils::Container for TestSoa {
    type Item = Test;
    fn with_capacity(capacity: usize) -> Self {
        Self {
            a: Vec::with_capacity(capacity),
            b: Vec::with_capacity(capacity),
            last: Vec::with_capacity(capacity),
        }
    }
    fn push(&mut self, value: Self::Item) {
        self.a.push(value.a);
        self.b.push(value.b);
        self.last.push(value.last);
    }
    fn pop(&mut self) -> Option<Self::Item> {
        Some(Self::Item {
            a: self.a.pop()?,
            b: self.b.pop()?,
            last: self.last.pop()?,
        })
    }
    fn swap_remove(&mut self, index: usize) -> Self::Item {
        Self::Item {
            a: self.a.swap_remove(index),
            b: self.b.swap_remove(index),
            last: self.last.swap_remove(index),
        }
    }
    fn replace(&mut self, index: usize, mut value: Self::Item) -> Self::Item {
        std::mem::swap(self.a.get_mut(index).unwrap(), &mut value.a);
        std::mem::swap(self.b.get_mut(index).unwrap(), &mut value.b);
        std::mem::swap(self.last.get_mut(index).unwrap(), &mut value.last);
        value
    }
    fn len(&self) -> usize {
        self.a.len()
    }
    fn capacity(&self) -> usize {
        self.a.capacity()
    }
    fn reserve(&mut self, additional: usize) {
        self.a.reserve(additional);
        self.b.reserve(additional);
        self.last.reserve(additional);
    }
    fn swap_elements(&mut self, a: usize, b: usize) {
        self.a.swap(a, b);
        self.b.swap(a, b);
        self.last.swap(a, b);
    }
}
