pub trait Incrementable: std::ops::AddAssign<Self> + Sized {
    fn one() -> Self;

    fn increment(&mut self) {
        self.add_assign(Self::one());
    }
}
macro_rules! impl_Incrementable{
    ($($type:ty),*) => {$( impl Incrementable for $type  { fn one() -> Self { 1 as $type } })*}
}
impl_Incrementable! {u8, u16, u32, u64, i8, i16, i32, i64, f32, f64}
