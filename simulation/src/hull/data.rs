use super::*;

#[derive(Debug)]
pub struct HullTexturePaths {
    pub albedo: &'static str,
    pub normal: Option<&'static str>,
}

#[derive(Debug)]
pub struct HullData {
    pub defence: Defence,
    pub shape: HullShape,
    pub density: f32,
    pub texture_paths: HullTexturePaths,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum HullDataId {
    Ball,
    Cuboid,
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
