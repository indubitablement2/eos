pub mod ai;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// If this entity is a ship from a fleet.
    pub fleet_ship: Option<(FleetId, usize)>,
    pub team: u32,

    pub entity_data_id: EntityDataId,

    pub rb: RigidBodyHandle,

    readiness: f32,
    mobility: Mobility,
    pub hulls: SmallVec<[Option<Hull>; 1]>,

    pub wish_angvel: WishAngVel,
    pub wish_linvel: WishLinVel,
    // pub wish_aim: (),
}
impl Entity {
    pub fn new(
        entity_data_id: EntityDataId,
        entity_id: EntityId,
        physics: &mut Physics,
        translation: na::Vector2<f32>,
        angle: f32,
    ) -> Entity {
        let entity_data = entity_data_id.data();

        let rb = physics.add_body(
            SimpleRigidBodyBuilder::dynamic()
                .translation(translation)
                .rotation(angle),
            BodyGenericId::EntityId(entity_id),
        );

        let hulls = entity_data
            .hulls
            .iter()
            .zip(0u32..)
            .map(|(hull_data, i)| {
                let collider = physics.add_collider(
                    SimpleColliderBuilder::new_ship(hull_data.shape.clone()),
                    rb,
                    ColliderGenericId::HullIndex(i),
                );

                Some(entity::Hull {
                    defence: hull_data.defence,
                    collider,
                })
            })
            .collect::<SmallVec<_>>();

        Self {
            fleet_ship: None,
            team: 0,
            entity_data_id,
            rb,
            mobility: entity_data_id.data().mobility,
            readiness: 1.0,
            hulls,
            wish_angvel: Default::default(),
            wish_linvel: Default::default(),
        }
    }

    pub fn result(&self) -> Option<bc_fleet::EntityResult> {
        if let Some(main_hull) = &self.hulls[0] {
            let max_defence = self.entity_data_id.data().hulls[0].defence;
            Some(bc_fleet::EntityResult {
                new_hull: main_hull.defence.hull as f32 / max_defence.hull as f32,
                new_armor: main_hull.defence.armor as f32 / max_defence.armor as f32,
                new_readiness: self.readiness,
            })
        } else {
            None
        }
    }

    pub fn compute_mobility(&mut self) {
        // TODO: Compute from buffs.
        self.mobility = self.entity_data_id.data().mobility;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishLinVel {
    /// Keep current velocity. eg. do nothing.
    #[default]
    Keep,
    /// Try to reach 0 linvel.
    Cancel,
    /// Always try to go forward at max velocity.
    Forward,
    /// Cancel our current velocity to reach position as fast as possible.
    /// Does not overshot.
    Position { position: na::Vector2<f32> },
    /// Same as position, but always try to go at max velocity.
    PositionOvershot { position: na::Vector2<f32> },
    Absolute {
        angle: na::UnitComplex<f32>,
        strenght: f32,
    },
    Relative {
        angle: na::UnitComplex<f32>,
        strenght: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishAngVel {
    /// Keep current angvel. eg. do nothing.
    #[default]
    Keep,
    /// Try to reach no angvel.
    Cancel,
    /// Set angvel to face world space position without overshot.
    Aim { position: na::Vector2<f32> },
    /// Set angvel to reach this rotation without overshot.
    Rotation(na::UnitComplex<f32>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hull {
    pub defence: Defence,
    pub collider: ColliderHandle,
}

pub struct EntityData {
    mobility: Mobility,
    /// First hull is main.
    pub hulls: SmallVec<[HullData; 1]>,
    // TODO: ai
    ai: Option<()>,
}
impl Default for EntityData {
    fn default() -> Self {
        Self {
            mobility: Default::default(),
            hulls: smallvec![Default::default()],
            ai: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Mobility {
    pub linear_acceleration: f32,
    pub angular_acceleration: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
}
impl Default for Mobility {
    fn default() -> Self {
        Self {
            linear_acceleration: 1.0,
            angular_acceleration: 0.5,
            max_linear_velocity: 7.0,
            max_angular_velocity: 3.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Defence {
    pub hull: i32,
    pub armor: i32,
}
impl Default for Defence {
    fn default() -> Self {
        Self {
            hull: 100,
            armor: 100,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HullData {
    pub defence: Defence,
    pub shape: SharedShape,
    pub density: f32,
    pub texture_paths: String,
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
}
impl Default for HullData {
    fn default() -> Self {
        Self {
            defence: Default::default(),
            shape: HullShape::default().to_shared_shape(),
            density: 1.0,
            texture_paths: Default::default(),
        }
    }
}

/// An entity data read from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDataTransient {
    mobility: Mobility,
    hulls: Vec<HullDataTransient>,
    ai: Option<()>,
}
impl EntityDataTransient {
    pub fn to_entity_data(mut self) -> EntityData {
        if self.hulls.is_empty() {
            self.hulls.push(Default::default());
        }
        let hulls = self
            .hulls
            .into_iter()
            .map(|hull| hull.to_hull_data())
            .collect::<SmallVec<_>>();

        EntityData {
            mobility: self.mobility,
            hulls,
            ai: self.ai,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HullDataTransient {
    defence: Defence,
    shape: HullShape,
    density: f32,
    texture_paths: String,
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
}
impl HullDataTransient {
    fn to_hull_data(self) -> HullData {
        HullData {
            defence: self.defence,
            shape: self.shape.to_shared_shape(),
            density: self.density,
            texture_paths: self.texture_paths,
        }
    }
}
impl Default for HullDataTransient {
    fn default() -> Self {
        Self {
            defence: Default::default(),
            shape: Default::default(),
            density: 1.0,
            texture_paths: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HullShape {
    Cuboid { hx: f32, hy: f32 },
    Ball { radius: f32 },
    Polygon { vertices: Vec<glam::Vec2> },
}
impl HullShape {
    pub fn to_shared_shape(&self) -> SharedShape {
        match self {
            HullShape::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            HullShape::Ball { radius } => SharedShape::ball(*radius),
            HullShape::Polygon { vertices } => {
                let vertices = vertices
                    .iter()
                    .map(|v| na::point![v.x, v.y])
                    .collect::<Vec<_>>();

                let indices = (0..vertices.len() as u32 - 1)
                    .map(|i| [i, i + 1])
                    .collect::<Vec<_>>();
                SharedShape::convex_decomposition(&vertices, indices.as_slice())
            }
        }
    }
}
impl Default for HullShape {
    fn default() -> Self {
        Self::Ball { radius: 0.5 }
    }
}
