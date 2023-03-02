pub mod ai;
pub mod script;
mod wishvel;

use self::script::*;
use self::wishvel::*;
use super::*;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    /// If this entity is a ship from a fleet.
    pub fleet_ship: Option<FleetShip>,
    pub owner: Option<ClientId>,
    pub team: u32,

    pub entity_data_id: EntityDataId,

    pub rb: RigidBodyHandle,
    pub hull_collider: ColliderHandle,

    pub readiness: f32,
    pub mobility: Mobility,
    pub defence: Defence,

    pub script: EntityScriptWrapper,

    pub wish_angvel: WishAngVel,
    pub wish_linvel: WishLinVel,
    // pub wish_aim: (),
    pub cb_destroyed: Callbacks,
}
impl Entity {
    pub fn new(
        entity_data_id: EntityDataId,
        fleet_ship: Option<FleetShip>,
        owner: Option<ClientId>,
        team: u32,
        rb: RigidBodyHandle,
        hull_collider: ColliderHandle,
        condition: EntityCondition,
    ) -> Entity {
        let entity_data = entity_data_id.data();

        let defence = Defence {
            hull: (entity_data.defence.hull as f32 * condition.hull) as i32,
            armor: (entity_data.defence.armor as f32 * condition.armor) as i32,
        };

        Self {
            fleet_ship,
            owner,
            team,
            entity_data_id,
            rb,
            hull_collider,
            mobility: entity_data.mobility,
            defence,
            readiness: condition.readiness,
            wish_angvel: Default::default(),
            wish_linvel: Default::default(),
            script: EntityScriptWrapper::new(entity_data_id),
            cb_destroyed: Default::default(),
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.defence.hull <= 0
    }

    pub fn condition(&self) -> EntityCondition {
        let max_defence = self.entity_data_id.data().defence;
        EntityCondition {
            hull: self.defence.hull as f32 / max_defence.hull as f32,
            armor: self.defence.armor as f32 / max_defence.armor as f32,
            readiness: self.readiness,
        }
    }

    fn compute_mobility(&mut self) {
        // TODO: Compute from buffs.
        self.mobility = self.entity_data_id.data().mobility;
    }

    /// Prepare the entity to be serialized.
    pub fn pre_serialize(&mut self) {
        self.script.prepare_serialize();
    }

    /// Prepare the entity post serialization.
    pub fn post_deserialize(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.post_deserialize(bs_ptr, entity_idx);
    }

    /// Should have called `post_deserialize` on all entity before this.
    pub fn post_post_deserialize(&mut self) {
        self.script.post_post_deserialize();
    }

    pub fn start(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.start(bs_ptr, entity_idx);
    }

    pub fn pre_step(&mut self, bs_ptr: BsPtr, entity_idx: usize) {
        self.script.prepare(bs_ptr, entity_idx);
    }

    /// Return if this should be removed.
    pub fn step(&mut self, physics: &mut Physics) -> bool {
        self.script.step();

        let rb = &mut physics.bodies[self.rb];

        // Angvel
        match self.wish_angvel {
            WishAngVel::Keep => {
                if rb.angvel() > self.mobility.max_angular_velocity {
                    let angvel = RealField::max(
                        angvel_stop(rb.angvel(), self.mobility.angular_acceleration),
                        self.mobility.max_angular_velocity,
                    );
                    rb.set_angvel(angvel, true);
                } else if rb.angvel() < -self.mobility.max_angular_velocity {
                    let angvel = RealField::min(
                        angvel_stop(rb.angvel(), self.mobility.angular_acceleration),
                        -self.mobility.max_angular_velocity,
                    );
                    rb.set_angvel(angvel, true);
                }
            }
            WishAngVel::Cancel => {
                let angvel = angvel_stop(rb.angvel(), self.mobility.angular_acceleration);
                rb.set_angvel(angvel, false);
            }
            WishAngVel::Aim { position } => {
                let target = position - *rb.translation();
                let wish_rot_offset = rb
                    .rotation()
                    .transform_vector(&na::Vector2::new(0.0, 1.0))
                    .angle_to(target);

                let angvel = angvel_target(
                    rb.angvel(),
                    wish_rot_offset,
                    self.mobility.angular_acceleration,
                    self.mobility.max_angular_velocity,
                );
                rb.set_angvel(angvel, true);
            }
            WishAngVel::Rotation(_) => {
                // TODO: Is rotation wish angvel useful?
                log::warn!("WishAngVel::Rotation not implemented yet.");
                let angvel = angvel_stop(rb.angvel(), self.mobility.angular_acceleration);
                rb.set_angvel(angvel, false);
            }
            WishAngVel::Force { force } => {
                let angvel = angvel_force(
                    rb.angvel(),
                    force,
                    self.mobility.angular_acceleration,
                    self.mobility.max_angular_velocity,
                );
                rb.set_angvel(angvel, true);
            }
        }

        // Linvel
        match self.wish_linvel {
            WishLinVel::Keep => {
                if rb.linvel().magnitude_squared() > self.mobility.max_linear_velocity * 1.02 {
                    // Slow down to max velocity.
                    let mut linvel = linvel_stop(*rb.linvel(), self.mobility.linear_acceleration);
                    if linvel.magnitude_squared() < self.mobility.max_linear_velocity {
                        // Slowed down too much, set to max velocity.
                        linvel.set_magnitude(self.mobility.max_linear_velocity);
                    }
                    rb.set_linvel(linvel, true);
                }
            }
            WishLinVel::Cancel => {
                let linvel = linvel_stop(*rb.linvel(), self.mobility.linear_acceleration);
                rb.set_linvel(linvel, false);
            }
            WishLinVel::Position { position } => {
                let target = position - rb.translation();
                if target.magnitude_squared() < 0.01 {
                    let linvel = linvel_stop(*rb.linvel(), self.mobility.linear_acceleration);
                    rb.set_linvel(linvel, false);
                } else {
                    let wish_vel = target.cap_magnitude(self.mobility.max_linear_velocity);
                    let linvel = linvel_wish_linvel(
                        *rb.linvel(),
                        wish_vel,
                        self.mobility.linear_acceleration,
                    );
                    rb.set_linvel(linvel, true);
                }
            }
            WishLinVel::PositionOvershot { position } => {
                let target = position - rb.translation();
                let wish_vel = if target.magnitude_squared() < 0.001 {
                    na::Vector2::new(0.0, self.mobility.max_linear_velocity)
                } else {
                    target.normalize().scale(self.mobility.max_linear_velocity)
                };
                let linvel =
                    linvel_wish_linvel(*rb.linvel(), wish_vel, self.mobility.linear_acceleration);
                rb.set_linvel(linvel, true);
            }
            WishLinVel::Absolute { force } => {
                let wish_vel = force * self.mobility.max_linear_velocity;
                let linvel =
                    linvel_wish_linvel(*rb.linvel(), wish_vel, self.mobility.linear_acceleration);
                rb.set_linvel(linvel, true);
            }
            WishLinVel::Relative { force } => {
                let wish_vel = rb
                    .rotation()
                    .transform_vector(&na::Vector2::new(-force.x, force.y))
                    * self.mobility.max_linear_velocity;
                let linvel =
                    linvel_wish_linvel(*rb.linvel(), wish_vel, self.mobility.linear_acceleration);
                rb.set_linvel(linvel, true);
            }
        }

        if self.is_destroyed() {
            self.cb_destroyed.emit();
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishLinVel {
    /// Keep current velocity. eg. do nothing.
    #[default]
    Keep,
    /// Try to reach 0 linvel.
    Cancel,
    /// Cancel our current velocity to reach position as fast as possible.
    /// Does not overshot.
    Position { position: na::Vector2<f32> },
    /// Same as position, but always try to go at max velocity.
    PositionOvershot { position: na::Vector2<f32> },
    /// `-y` is up.
    Absolute {
        /// Magnitude 0 to 1
        force: na::Vector2<f32>,
    },
    /// Relative to current rotation.
    /// `+y` is forward.
    Relative {
        /// Magnitude 0 to 1
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

pub struct EntityData {
    pub mobility: Mobility,
    /// `EntityScript`
    pub script: EntityScriptData,
    pub defence: Defence,
    pub collider: Collider,
    // TODO: Expand to other type like drone, station, etc.
    pub is_ship: bool,
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: A Shields
}
impl Default for EntityData {
    fn default() -> Self {
        Self {
            mobility: Default::default(),
            script: Default::default(),
            defence: Default::default(),
            collider: ball_collider(0.5, 1.0, Groups::Ship),
            is_ship: false,
        }
    }
}
impl std::fmt::Debug for EntityData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityData")
            .field("mobility", &self.mobility)
            .field("script", &self.script)
            .field("defence", &self.defence)
            .field("shape", &self.collider.shape().shape_type())
            .finish()
    }
}

/// In unit/seconds.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Mobility {
    /// In unit/tick.
    pub linear_acceleration: f32,
    /// In radian/tick.
    pub angular_acceleration: f32,
    /// In unit/seconds.
    pub max_linear_velocity: f32,
    /// In radian/seconds.
    pub max_angular_velocity: f32,
}
impl Default for Mobility {
    fn default() -> Self {
        Self {
            linear_acceleration: 1.0,
            angular_acceleration: 1.0,
            max_linear_velocity: 2.0,
            max_angular_velocity: 2.0,
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

/// In absolute value 0..1
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityCondition {
    pub hull: f32,
    pub armor: f32,
    pub readiness: f32,
}
impl EntityCondition {
    pub fn full() -> Self {
        Self {
            hull: 1.0,
            armor: 1.0,
            readiness: 1.0,
        }
    }

    pub fn empty() -> Self {
        Self {
            hull: 0.0,
            armor: 0.0,
            readiness: 0.0,
        }
    }
}
impl Default for EntityCondition {
    fn default() -> Self {
        Self::full()
    }
}
