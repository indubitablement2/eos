use super::*;
use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum HullDataId {
    Ball,
    Cuboid
}
impl HullDataId {
    pub const fn data(self) -> HullData {
        match self {
            Self::Ball => HullData {
                defence: Defence {
                    hull: 100,
                    armor: 100,
                },
                shape: HullShape::Ball { radius: 0.5 },
                density: 1.0,
                texture_paths: HullTexturePaths {
                    albedo: "res://assets/debug/colored_circle128.png",
                    normal: None,
                },
            },
            Self::Cuboid => HullData {
                defence: Defence {
                    hull: 100,
                    armor: 100,
                },
                shape: HullShape::Cuboid { hx: 0.5, hy: 0.5 },
                density: 1.0,
                texture_paths: HullTexturePaths {
                    albedo: "res://assets/debug/PixelTextureGrid_128.png",
                    normal: None,
                },
            },
        }
    }
}

#[derive(Debug)]
pub struct HullData {
    pub defence: Defence,
    pub shape: HullShape,
    pub density: f32,
    pub texture_paths: HullTexturePaths,
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

#[derive(Debug)]
pub struct HullTexturePaths {
    pub albedo: &'static str,
    pub normal: Option<&'static str>,
}
