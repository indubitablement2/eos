pub mod ai;
mod script;

use godot::prelude::{Gd, Node2D};
use self::script::ScriptWrapper;
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

    pub script: ScriptWrapper,

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
            script: todo!(),
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
    pub mobility: Mobility,
    /// First hull is main.
    pub hulls: SmallVec<[HullData; 1]>,
    // TODO: ai
    pub ai: Option<()>,
    pub node: Gd<Node2D>,
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
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
    /// Node index as child of an entity.
    pub node_idx: i64,
}
