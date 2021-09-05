use rand::prelude::*;
use simdnoise::*;

/// Input: Dimention of the chunk.
///
/// Output: Array of ground tile.
pub fn generate(width: usize, height: usize) -> Vec<u8> {
    let num_tiles = width * height;

    // Generate noise.
    let noise = NoiseBuilder::cellular2_2d(width, height)
        .with_seed(random())
        .with_freq(0.1)
        .generate_scaled(0.0, 1.0);

    // Make chunk from noise.
    let mut result = Vec::with_capacity(num_tiles);
    noise.into_iter().for_each(|v| {
        if v <= 0.3 {
            result.push(1);
        } else if v <= 0.5 {
            result.push(2);
        } else if v <= 0.6 {
            result.push(3);
        }
         else {
            result.push(0);
        }
    });

    result
}
