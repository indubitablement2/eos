pub mod ai;
mod script;

use self::script::*;
use super::*;
use godot::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    /// If this entity is a ship from a fleet.
    pub fleet_ship: Option<(FleetId, usize)>,
    pub team: u32,

    pub entity_data_id: EntityDataId,

    pub rb: RigidBodyHandle,

    readiness: f32,
    mobility: Mobility,
    pub hulls: SmallVec<[Option<Hull>; 1]>,

    pub script: EntityScriptWrapper,

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
            .map(|(hull_data, hull_idx)| {
                let collider = physics.add_collider(
                    SimpleColliderBuilder::new_ship(hull_data.shape.clone())
                        .density(hull_data.density)
                        .position(hull_data.init_position),
                    rb,
                    ColliderGenericId::Hull {
                        entity_id,
                        hull_idx,
                    },
                );

                Some(entity::Hull {
                    defence: hull_data.defence,
                    collider,
                    script: HullScriptWrapper::new(entity_data_id, hull_idx),
                })
            })
            .collect::<SmallVec<_>>();

        Self {
            fleet_ship: None,
            team: 0,
            entity_data_id,
            rb,
            mobility: entity_data.mobility,
            readiness: 1.0,
            hulls,
            wish_angvel: Default::default(),
            wish_linvel: Default::default(),
            script: EntityScriptWrapper::new(entity_data_id),
        }
    }

    pub fn result(&self) -> Option<bc_fleet::EntityResult> {
        self.hulls[0].as_ref().map(|main_hull| {
            let max_defence = self.entity_data_id.data().hulls[0].defence;
            bc_fleet::EntityResult {
                new_hull: main_hull.defence.hull as f32 / max_defence.hull as f32,
                new_armor: main_hull.defence.armor as f32 / max_defence.armor as f32,
                new_readiness: self.readiness,
            }
        })
    }

    fn compute_mobility(&mut self) {
        // TODO: Compute from buffs.
        self.mobility = self.entity_data_id.data().mobility;
    }

    /// Prepare the entity to be serialized.
    pub fn pre_serialize(&mut self) {
        self.script.pre_serialize();
        for hull in self.hulls.iter_mut() {
            if let Some(hull) = hull {
                hull.script.pre_serialize();
            }
        }
    }

    /// Prepare the entity post serialization.
    pub fn post_deserialize_prepare(&mut self, bc_ptr: Variant, entity_idx: Variant) {
        self.script
            .post_deserialize_prepare(bc_ptr.to_variant(), entity_idx.to_variant());
        for (hull, hull_idx) in self.hulls.iter_mut().zip(0..) {
            if let Some(hull) = hull {
                hull.script.post_deserialize_prepare(
                    bc_ptr.to_variant(),
                    entity_idx.to_variant(),
                    hull_idx,
                );
            }
        }
    }

    /// Should have called `post_deserialize_prepare` on all entity before this.
    pub fn post_deserialize_post_prepare(&mut self) {
        self.script.post_deserialize_post_prepare();
        for hull in self.hulls.iter_mut() {
            if let Some(hull) = hull {
                hull.script.post_deserialize_post_prepare();
            }
        }
    }

    pub fn prepare_script(&mut self, bc_ptr: i64, entity_idx: i64) {
        self.script
            .prepare(bc_ptr.to_variant(), entity_idx.to_variant());
        for (hull, hull_idx) in self.hulls.iter_mut().zip(0i64..) {
            if let Some(hull) = hull {
                hull.script.prepare(
                    bc_ptr.to_variant(),
                    entity_idx.to_variant(),
                    hull_idx.to_variant(),
                );
            }
        }
    }

    pub fn step_script(&mut self) {
        self.script.step();
        for hull in self.hulls.iter_mut() {
            if let Some(hull) = hull {
                hull.script.step();
            }
        }
    }

    pub fn step(&mut self) {}
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Hull {
    pub defence: Defence,
    pub collider: ColliderHandle,
    pub script: HullScriptWrapper,
}

pub struct EntityData {
    pub mobility: Mobility,
    /// First hull is main.
    pub hulls: SmallVec<[HullData; 1]>,
    // TODO: ai
    pub ai: Option<()>,
    pub render_node: Gd<Node2D>,
    /// `EntityScript`
    pub script: Variant,
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

pub struct HullData {
    pub defence: Defence,
    pub shape: SharedShape,
    pub init_position: Isometry<Real>,
    pub density: f32,
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
    pub render_node_idx: i64,
    /// `HullScript`
    pub script: Variant,
}
