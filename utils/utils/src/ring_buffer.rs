use num_traits::{NumAssignOps, PrimInt};

pub trait RingBufferTrait {
    type Item;
    /// Set the next value in the buffer
    /// and return if we are back at the beginning.
    fn set_next(&mut self, value: Self::Item) -> bool;
    fn fold(&self) -> RingBufferFold<Self::Item>;
}

/// Result of `fold()` on a ring buffer.
#[derive(Debug, Default, PartialEq)]
pub struct RingBufferFold<T> {
    pub sum: T,
    pub avg: T,
    pub min: T,
    pub max: T,
}

/// Ring buffer of `f32` with a generic size.
#[derive(Debug)]
pub struct RingBufferF<const N: usize> {
    pub buffer: [f32; N],
    cursor: usize,
}
impl<const N: usize> RingBufferTrait for RingBufferF<N> {
    type Item = f32;

    fn set_next(&mut self, value: Self::Item) -> bool {
        self.buffer[self.cursor] = value;
        self.cursor = (self.cursor + 1) % N;
        self.cursor == 0
    }

    fn fold(&self) -> RingBufferFold<Self::Item> {
        let first = *self.buffer.first().unwrap();
        let mut fold: RingBufferFold<f32> = RingBufferFold {
            sum: first,
            avg: 0.0,
            min: first,
            max: first,
        };
        self.buffer[1..].iter().for_each(|&value| {
            fold.sum += value;
            fold.min = fold.min.min(value);
            fold.max = fold.max.max(value);
        });
        fold.avg = fold.sum / N as f32;
        fold
    }
}
impl<const N: usize> Default for RingBufferF<N> {
    fn default() -> Self {
        Self {
            buffer: [Default::default(); N],
            cursor: Default::default(),
        }
    }
}

/// Ring buffer of `int` with a generic size.
#[derive(Debug)]
pub struct RingBufferI<T, const N: usize>
where
    T: Default + PrimInt + NumAssignOps,
{
    pub buffer: [T; N],
    cursor: usize,
}
impl<T, const N: usize> RingBufferTrait for RingBufferI<T, N>
where
    T: Default + PrimInt + NumAssignOps,
{
    type Item = T;

    fn set_next(&mut self, value: Self::Item) -> bool {
        self.buffer[self.cursor] = value;
        self.cursor = (self.cursor + 1) % N;
        self.cursor == 0
    }

    fn fold(&self) -> RingBufferFold<Self::Item> {
        let first = *self.buffer.first().unwrap();
        let mut fold: RingBufferFold<T> = RingBufferFold {
            sum: first,
            avg: T::zero(),
            min: first,
            max: first,
        };
        self.buffer[1..].iter().for_each(|&value| {
            fold.sum += value;
            fold.min = fold.min.min(value);
            fold.max = fold.max.max(value);
        });
        fold.avg = fold.sum / num_traits::cast(N).unwrap();
        fold
    }
}
impl<T, const N: usize> Default for RingBufferI<T, N>
where
    T: Default + PrimInt + NumAssignOps,
{
    fn default() -> Self {
        Self {
            buffer: [Default::default(); N],
            cursor: Default::default(),
        }
    }
}

#[test]
fn test_ring_buffer_float() {
    let mut r: RingBufferF<10> = RingBufferF::default();

    for i in 0..9 {
        r.set_next(i as f32);
    }
    assert!(r.set_next(9.0));
    assert!(!r.set_next(10.0));

    assert_eq!(
        r.fold(),
        RingBufferFold {
            sum: 55.0f32,
            avg: 5.5,
            min: 1.0,
            max: 10.0
        }
    )
}

#[test]
fn test_ring_buffer_generic_int() {
    let mut r: RingBufferI<i32, 10> = RingBufferI::default();

    for i in 0..9 {
        r.set_next(i);
    }
    assert!(r.set_next(9));
    assert!(!r.set_next(10));

    assert_eq!(
        r.fold(),
        RingBufferFold {
            sum: 55,
            avg: 5,
            min: 1,
            max: 10
        }
    )
}
