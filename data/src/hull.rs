use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct HullData {
    pub defence: Defence,
    pub shape: HullShape,
    pub density: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Defence {
    pub hull: i32,
    pub armor: i32,
}

#[derive(Debug)]
pub enum HullShape {
    Cuboid {
        hx: f32,
        hy: f32,
    },
    Ball {
        radius: f32,
    },
    Polygon {
        vertices: &'static [na::Point2<f32>],
    },
}
impl HullShape {
    pub fn to_shared_shape(&self) -> SharedShape {
        match self {
            HullShape::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            HullShape::Ball { radius } => SharedShape::ball(*radius),
            HullShape::Polygon { vertices } => {
                // TODO: Precompute this.
                let indices = (0..vertices.len() as u32 - 1)
                    .map(|i| [i, i + 1])
                    .collect::<Vec<_>>();
                SharedShape::convex_decomposition(vertices, indices.as_slice())
            }
        }
    }
}
