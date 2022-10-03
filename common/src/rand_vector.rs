use std::ops::Range;
use rand::prelude::*;

/// Return a random vector with independant x and y value.
pub fn rand_vec2<R>(rng: &mut R, range: Range<f32>) -> na::Vector2<f32>
where
    R: Rng,
{
    na::vector![rng.gen_range(range.clone()), rng.gen_range(range)]
}
