

#[derive(Debug, Default, PartialEq)]
pub struct RingBufferFold<T> {
    pub sum: T,
    pub avg: T,
    pub min: T,
    pub max: T,
}

#[derive(Debug)]
pub struct RingBufferF<const N: usize> {
    pub buffer: [f32; N],
    cursor: usize,
}
impl<const N: usize> RingBufferF<N> {
    pub fn set_next(&mut self, value: f32) {
        self.buffer[self.cursor] = value;
        self.cursor = (self.cursor + 1) % N;
    }

    pub fn fold(&self) -> RingBufferFold<f32> {
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
impl<const N: usize> Default for RingBufferF<N>
{
    fn default() -> Self {
        Self { buffer: [Default::default(); N], cursor: Default::default() }
    }
}

#[test]
fn asd() {
    let mut r: RingBufferF<10> = RingBufferF::default();

    for i in 0..11 {
        r.set_next(i as f32);
    }

   assert_eq!(r.fold(), RingBufferFold { sum: 55.0f32, avg: 5.5, min: 1.0, max: 10.0 })
}