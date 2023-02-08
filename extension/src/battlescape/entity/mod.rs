pub mod ai;
pub mod script;

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
                    script: HullScriptWrapper::new(entity_data_id, hull_idx as usize),
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

    pub fn is_destroyed(&mut self) -> bool {
        self.hulls[0].is_none()
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
    pub fn post_deserialize_prepare(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.post_deserialize_prepare(bs_ptr, entity_idx);
        for (hull, hull_idx) in self.hulls.iter_mut().zip(0..) {
            if let Some(hull) = hull {
                hull.script
                    .post_deserialize_prepare(bs_ptr, entity_idx, hull_idx);
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

    pub fn pre_step(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.prepare(bs_ptr, entity_idx);
        for (hull, hull_idx) in self.hulls.iter_mut().zip(0usize..) {
            if let Some(hull) = hull {
                hull.script.prepare(bs_ptr, entity_idx, hull_idx);
            }
        }
    }

    pub fn step(&mut self, physics: &mut Physics) {
        // Scripts
        self.script.step();
        for hull in self.hulls.iter_mut() {
            if let Some(hull) = hull {
                hull.script.step();
            }
        }

        let rb = &mut physics.bodies[self.rb];

        // Angvel
        match self.wish_angvel {
            WishAngVel::Keep => {}
            WishAngVel::Cancel => {
                if ComplexField::abs(rb.angvel()) > 0.0001 {
                    let new_angvel = rb.angvel()
                        - RealField::clamp(
                            rb.angvel(),
                            -self.mobility.angular_acceleration,
                            self.mobility.angular_acceleration,
                        );
                    rb.set_angvel(new_angvel, false);
                }
            }
            WishAngVel::Aim { position } => {
                let target = position - *rb.translation();
                let target_angle = target.angle_x();
                let wish_rot_offset = target_angle - rb.rotation().angle();

                let angvel_change = RealField::clamp(
                    wish_rot_offset,
                    -self.mobility.angular_acceleration,
                    self.mobility.angular_acceleration,
                );

                // TODO: Angvel cap.
                let wish_new_angvel = rb.angvel() + angvel_change;

                rb.set_angvel(wish_new_angvel, true);
            }
            WishAngVel::Rotation(_) => {
                // TODO:
            }
            WishAngVel::Force { force } => {
                // TODO:
            }
        }

        //     fn apply_wish_linvel(
        //         wish_vel: Vector2,
        //         rb: &mut RigidBody,
        //         linear_acceleration: f32,
        //         wake_up: bool,
        //     ) {
        //         let vel_change = (wish_vel - rb.linvel()).cap_magnitude(linear_acceleration * DT);
        //         rb.set_linvel(rb.linvel() + vel_change, wake_up);
        //     }

        //     // Linvel
        //     match e.wish_linvel {
        //         WishLinVel::Keep => {}
        //         WishLinVel::Cancel => {
        //             if rb.linvel().magnitude_squared() > 0.001 {
        //                 apply_wish_linvel(-rb.linvel(), rb, e.mobility.linear_acceleration, false);
        //                 // rb.set_linvel(
        //                 //     rb.linvel() - rb.linvel().cap_magnitude(e.mobility.linear_acceleration * DT),
        //                 //     false,
        //                 // );
        //             }
        //         }
        //         WishLinVel::Forward => {
        //             let wish_vel = rb
        //                 .rotation()
        //                 .transform_vector(&na::vector![0.0, e.mobility.max_linear_velocity]);
        //             apply_wish_linvel(wish_vel, rb, e.mobility.linear_acceleration, true);
        //         }
        //         WishLinVel::Position { position } => {
        //             // TODO:
        //         }
        //         WishLinVel::PositionOvershot { position } => {
        //             // TODO:
        //         }
        //         WishLinVel::Absolute { angle, strenght } => {
        //             let wish_vel = angle.transform_vector(&na::vector![
        //                 0.0,
        //                 e.mobility.max_linear_velocity * strenght
        //             ]);
        //             apply_wish_linvel(wish_vel, rb, e.mobility.linear_acceleration, true);
        //         }
        //         WishLinVel::Relative { angle, strenght } => {
        //             let wish_vel =
        //                 rb.rotation()
        //                     .transform_vector(&angle.transform_vector(&na::vector![
        //                         0.0,
        //                         e.mobility.max_linear_velocity * strenght
        //                     ]));
        //             apply_wish_linvel(wish_vel, rb, e.mobility.linear_acceleration, true);
        //         }
        //     }

        //     // let linvel: Vector2 = *rb.linvel();
        //     // let wish_linvel = if let Some(wish_pos) = e.wish_pos {
        //     //     let linvel_magnitude = linvel.magnitude();
        //     //     let time_to_break = linvel_magnitude / (e.mobility.linear_acceleration * 1.0);

        //     //     let relative_target = wish_pos - pos.translation.vector;

        //     //     let wish_vel = relative_target - linvel * time_to_break;
        //     //     wish_vel.cap_magnitude(e.mobility.max_linear_velocity)
        //     // } else {
        //     //     na::Vector2::zeros()
        //     // };
        //     // let linvel = (wish_linvel - linvel).cap_magnitude(e.mobility.linear_acceleration);
        //     // rb.set_linvel(linvel, true);
        // }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishLinVel {
    /// Keep current velocity. eg. do nothing.
    #[default]
    Keep,
    /// Try to reach 0 linvel.
    Cancel,
    /// Always try to go forward (or backward with negative force)
    /// at percent of max acceleration [-1..1].
    Forward {
        force: f32,
    },
    /// Cancel our current velocity to reach position as fast as possible.
    /// Does not overshot.
    Position {
        position: na::Vector2<f32>,
    },
    /// Same as position, but always try to go at max velocity.
    PositionOvershot {
        position: na::Vector2<f32>,
    },
    Absolute {
        force: na::Vector2<f32>,
    },
    /// Relative to current rotation.
    Relative {
        force: na::Vector2<f32>,
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
    /// Rotate left or right [-1..1].
    Force { force: f32 },
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
    /// Node2D
    pub render_node: Gd<PackedScene>,
    /// `EntityScript`
    pub script: Variant,
}

/// In unit/seconds.
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
