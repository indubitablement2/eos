use super::*;
use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct HullData {
    pub mobility: Mobility,
    pub defence: Defence,
    pub shape: HullShape,
    pub density: f32,
    /// Any hull normally bundled as child of this.
    /// TODO: Add: join, position offset, init_linvel, init_angvel
    pub child_hulls: &'static [(HullDataId, ())],
    /// Its memberships and what memberships can this hull collide with.
    pub groups: InteractionGroups,
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Mobility {
    pub linear_acceleration: f32,
    pub angular_acceleration: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
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

pub const GROUP_SHIP: Group = Group::GROUP_1;
pub const GROUP_SHIELD: Group = Group::GROUP_2;
pub const GROUP_DEBRIS: Group = Group::GROUP_3;
pub const GROUP_MISSILE: Group = Group::GROUP_4;
pub const GROUP_FIGHTER: Group = Group::GROUP_5;
pub const GROUP_PROJECTILE: Group = Group::GROUP_6;
pub const GROUP_ALL: Group = GROUP_SHIP
    .union(GROUP_SHIELD)
    .union(GROUP_DEBRIS)
    .union(GROUP_MISSILE)
    .union(GROUP_FIGHTER)
    .union(GROUP_PROJECTILE);
const PRESET_GROUPS_SHIP: InteractionGroups = InteractionGroups::new(GROUP_SHIP, GROUP_ALL);

pub const HULLS: &[HullData] = &[
    // 0
    HullData {
        mobility: Mobility {
            linear_acceleration: 1.0,
            angular_acceleration: 1.0,
            max_linear_velocity: 1.0,
            max_angular_velocity: 1.0,
        },
        defence: Defence {
            hull: 100,
            armor: 100,
        },
        shape: HullShape::Ball { radius: 1.0 },
        density: 1.0,
        child_hulls: &[],
        groups: PRESET_GROUPS_SHIP,
    },
    // 1
    HullData {
        mobility: Mobility {
            linear_acceleration: 1.0,
            angular_acceleration: 1.0,
            max_linear_velocity: 1.0,
            max_angular_velocity: 1.0,
        },
        defence: Defence {
            hull: 100,
            armor: 100,
        },
        shape: HullShape::Cuboid { hx: 0.5, hy: 0.5 },
        density: 1.0,
        child_hulls: &[],
        groups: PRESET_GROUPS_SHIP,
    },
];
