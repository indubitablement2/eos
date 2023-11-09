use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_data_id: EntityDataId,

    pub rb: RigidBodyHandle,

    pub defence: Defence,
    mobility: Mobility,

    pub target: Option<EntityId>,

    pub wish_angvel: WishAngVel,
    pub wish_linvel: WishLinVel,
    // pub wish_aim: (),
}
impl Entity {
    pub fn new(
        entity_data_id: EntityDataId,
        entity_id: EntityId,
        physics: &mut Physics,
        position: na::Isometry2<f32>,
    ) -> Entity {
        let entity_data = entity_data_id.data();

        let rb = physics.add_body(
            SimpleRigidBodyBuilder::dynamic()
                .translation(position.translation.vector)
                .rotation(position.rotation.angle()),
            BodyGenericId::EntityId(entity_id),
        );

        physics.add_collider(
            SimpleColliderBuilder::new_ship(entity_data.shape.clone()),
            rb,
            ColliderGenericId::HullIndex(0),
        );

        Self {
            entity_data_id,
            rb,
            defence: entity_data.defence,
            mobility: entity_data.mobility,
            wish_angvel: Default::default(),
            wish_linvel: Default::default(),
            target: None,
        }
    }

    /// Returns `true` if the entity was destroyed.
    pub fn update(&mut self, physics: &mut Physics) -> bool {
        let rb = physics.bodies.get_mut(self.rb).unwrap();
        let angvel = rb.angvel();
        let linvel = *rb.linvel();

        let wish_angvel = match self.wish_angvel {
            WishAngVel::None => angvel,
            WishAngVel::Keep => angvel.clamp(
                -self.mobility.max_angular_velocity,
                self.mobility.max_angular_velocity,
            ),
            WishAngVel::Stop => 0.0,
            WishAngVel::AimSmooth(aim_to) => {
                // TODO: May need to rotate this.
                // aim_to.angle(other)
                // let offset = Vec2::from_angle(angle).angle_between(to);

                // let offset = angle_to(rotation.0, *aim_to - position.0);
                // let wish_dir = offset.signum();
                // let mut close_smooth = offset.abs().min(0.2) / 0.2;
                // close_smooth *= close_smooth * close_smooth;

                // if wish_dir == angular.velocity.signum() {
                //     let time_to_target = (offset / angular.velocity).abs();
                //     let time_to_stop = (angular.velocity / (angular.acceleration)).abs();
                //     if time_to_target < time_to_stop {
                //         close_smooth *= -1.0;
                //     }
                // }

                // angular.velocity = integrate_angular_velocity(
                //     angular.velocity,
                //     wish_dir * angular.max_velocity * close_smooth,
                //     angular.acceleration,
                //     time.dt,
                // );
                0.0
            }
            WishAngVel::Force(force) => force * self.mobility.max_angular_velocity,
        };

        let wish_linvel = match self.wish_linvel {
            WishLinVel::None => linvel,
            WishLinVel::Keep => linvel.cap_magnitude(self.mobility.max_linear_velocity),
            WishLinVel::Cancel => vector![0.0, 0.0],
            WishLinVel::PositionSmooth(position) => {
                let to_position = position - rb.translation();
                if to_position.magnitude_squared() < 0.01 {
                    vector![0.0, 0.0]
                } else {
                    to_position.cap_magnitude(self.mobility.max_linear_velocity)
                }
            }
            WishLinVel::PositionOvershoot(position) => {
                (position - rb.translation())
                    .try_normalize(0.01)
                    .unwrap_or(vector![0.0, -1.0])
                    * self.mobility.max_linear_velocity
            }
            WishLinVel::ForceAbsolute(force) => force * self.mobility.max_linear_velocity,
            WishLinVel::ForceRelative(force) => {
                rb.rotation().transform_vector(&force) * self.mobility.max_linear_velocity
            }
        };

        let wake_up = wish_angvel != angvel || wish_linvel != linvel;
        rb.set_angvel(
            angvel
                + (wish_angvel - angvel).clamp(
                    -self.mobility.angular_acceleration * DT,
                    self.mobility.angular_acceleration * DT,
                ),
            wake_up,
        );
        rb.set_linvel(
            linvel + (wish_linvel - linvel).cap_magnitude(self.mobility.linear_acceleration),
            wake_up,
        );

        self.defence.hull <= 0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishAngVel {
    /// Do nothing.
    #[default]
    None,
    /// Keep current angvel unless above max.
    Keep,
    /// Try to reach 0 angvel.
    Stop,
    /// Set angvel to face world space position without overshot.
    AimSmooth(Vector2<f32>),
    /// Turn left or right.
    /// **Force should be clamped to 1**
    Force(f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum WishLinVel {
    /// Do nothing.
    #[default]
    None,
    /// Keep current linvel unless above max.
    Keep,
    /// Try to reach 0 linvel.
    Cancel,
    /// Cancel our current velocity to reach position as fast as possible.
    /// Does not overshot.
    PositionSmooth(Vector2<f32>),
    /// Move toward target at max velocity.
    /// When very close to target, goes upward.
    PositionOvershoot(Vector2<f32>),
    /// A force in world space. -y is up.
    /// **Force magnitude should be clamped to 1**
    ForceAbsolute(Vector2<f32>),
    /// A force in local space. -y is left, +x is forward.
    /// **Force magnitude should be clamped to 1**
    ForceRelative(Vector2<f32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityDataId(pub u32);
impl EntityDataId {
    pub fn data(self) -> &'static EntityData {
        unsafe { &ENTITY_DATA[self.0 as usize] }
    }
}

static mut ENTITY_DATA: Vec<EntityData> = Vec::new();

pub struct EntityData {
    pub defence: Defence,
    pub shape: SharedShape,
    pub density: f32,

    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
    pub mobility: Mobility,

    pub ai: Option<EntityAi>,
}
impl EntityData {
    pub fn set_data(data: Vec<Self>) {
        unsafe {
            ENTITY_DATA = data;
        }
    }

    pub fn get_data() -> &'static [Self] {
        unsafe { &ENTITY_DATA }
    }
}
impl Default for EntityData {
    fn default() -> Self {
        Self {
            defence: Default::default(),
            shape: HullShape::default().to_shared_shape(),
            density: 1.0,
            mobility: Default::default(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HullShape {
    Cuboid { hx: f32, hy: f32 },
    Ball { radius: f32 },
    Polygon { vertices: Vec<f32> },
}
impl HullShape {
    pub fn to_shared_shape(&self) -> SharedShape {
        match self {
            HullShape::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            HullShape::Ball { radius } => SharedShape::ball(*radius),
            HullShape::Polygon { vertices } => {
                let vertices = vertices
                    .chunks_exact(2)
                    .map(|v| na::point![v[0], v[1]])
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

#[test]
fn test_rotation() {
    let a_translation = vector![100.0f32, 200.0];
    let a_rotation = f32::to_radians(35.0);
    let mut body = RigidBodyBuilder::dynamic().build();
    body.set_translation(a_translation, true);
    body.set_rotation(na::UnitComplex::new(a_rotation), true);

    let b_translation = point![-50.0f32, 70.0];
    let b_rotation = f32::to_radians(45.0);
    let b_position = na::Isometry2::new(b_translation.coords, b_rotation);

    let target = point![350.0f32, 300.0];

    let global_translation = body.position().transform_point(&b_translation);
    let global_position = body.position() * b_position;
    let global_rotation = global_position.rotation.angle();
    let rotation_to_target =
        global_position
            .rotation
            .rotation_to(&na::UnitComplex::rotation_between(
                &vector![1.0, 0.0],
                &(target.coords - global_position.translation.vector),
            ));

    println!("{:?}", global_translation);
    println!("{:?}", global_rotation);
    println!("{:?}", rotation_to_target.angle());
}
