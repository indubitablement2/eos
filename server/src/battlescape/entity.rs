use super::*;

/// `[0..1]` relative to armor_hp_max.
type ArmorCells = SmallVec<[u8; 16]>;

pub struct Entity {
    pub data: &'static EntityData,

    pub rb: RigidBodyHandle,

    hull_max: f32,
    hull: f32,

    armor_max: f32,
    armor_cells: ArmorCells,

    mobility: Mobility,

    pub wish_angvel: WishAngVel,
    pub wish_linvel: WishLinVel,
    // pub wish_aim: (),
    pub controlled: bool,

    pub target: Option<EntityId>,
    // TODO: Events (hit, leaving, death, etc)
}
impl Entity {
    // TODO: from save
    pub fn new(
        battlescape: &mut Battlescape,

        data: &'static EntityData,
        mut save: EntitySave,

        entity_id: EntityId,

        ignore: Option<EntityId>,
        target: Option<EntityId>,
    ) -> Entity {
        let rb = battlescape.physics.add_body(
            save.position,
            save.linvel,
            save.angvel,
            data.shape.clone(),
            data.groups,
            data.mprops,
            entity_id,
            ignore,
        );

        save.armor_cells.resize(
            data.armor_cells_size.x as usize * data.armor_cells_size.y as usize,
            0,
        );

        let mut s = Self {
            data,
            rb,
            hull_max: data.hull_max,
            hull: save.hull,
            armor_max: data.armor_max,
            armor_cells: save.armor_cells,
            mobility: data.mobility,
            wish_angvel: WishAngVel::None,
            wish_linvel: WishLinVel::None,
            controlled: false,
            target,
        };

        for new_event in data.on_new.iter() {
            match new_event {
                EntityEvent::Ship => {
                    battlescape.objects.push(Object::Ship { entity_id });
                }
                EntityEvent::Seek => {
                    battlescape
                        .objects
                        .push(Object::new_seek(&mut s, entity_id));
                }
            }
        }

        s
    }

    pub fn take_contact_event(&mut self, event: ContactEvent) {
        // TODO
    }

    /// Returns `true` if the entity was destroyed.
    pub fn update(&mut self, physics: &mut Physics) -> bool {
        let rb = physics.body_mut(self.rb);
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

        self.hull < 0.0
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

// ####################################################################################
// ################################### DATA ###########################################
// ####################################################################################

pub struct EntityData {
    pub id: EntityDataId,

    hull_max: f32,

    armor_max: f32,
    armor_cells_size: Vector2<i32>,
    /// The maximum value a cell can have.
    armor_cells: ArmorCells,

    shape: SharedShape,
    mprops: MassProperties,
    groups: InteractionGroups,

    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
    pub mobility: Mobility,

    on_new: Vec<EntityEvent>,
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

#[derive(Debug, Serialize, Deserialize)]
enum EntityEvent {
    Ship,
    Seek,
}

// ####################################################################################
// ############################## DATA JSON ###########################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EntityDataJson {
    hull: f32,

    armor_max: f32,
    armor_cells_size: Vector2<i32>,
    armor_cells: ArmorCells,

    shape: HullShapeJson,
    mass_radius: f32,
    density: f32,
    groups: InteractionGroups,

    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
    mobility: Mobility,

    on_new: Vec<EntityEvent>,
}
impl EntityDataJson {
    pub fn parse(self, id: EntityDataId) -> EntityData {
        EntityData {
            id,

            hull_max: self.hull,

            armor_max: self.armor_max,
            armor_cells_size: self.armor_cells_size,
            armor_cells: self.armor_cells,

            shape: self.shape.to_shared_shape(),
            mprops: MassProperties::from_ball(self.density, self.mass_radius),
            groups: self.groups,

            mobility: self.mobility,

            on_new: self.on_new,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HullShapeJson {
    Cuboid { hx: f32, hy: f32 },
    Ball { radius: f32 },
    Polygon { vertices: Vec<f32> },
}
impl HullShapeJson {
    fn to_shared_shape(&self) -> SharedShape {
        match self {
            HullShapeJson::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            HullShapeJson::Ball { radius } => SharedShape::ball(*radius),
            HullShapeJson::Polygon { vertices } => {
                let vertices = vertices
                    .chunks_exact(2)
                    .map(|v| na::point![v[0], v[1]])
                    .collect::<Vec<_>>();

                let indices = (0..vertices.len() as u32 - 1)
                    .map(|i| [i, i + 1])
                    .chain(std::iter::once([vertices.len() as u32 - 1, 0]))
                    .collect::<Vec<_>>();
                SharedShape::convex_decomposition(&vertices, indices.as_slice())
            }
        }
    }
}
impl Default for HullShapeJson {
    fn default() -> Self {
        Self::Ball { radius: 0.5 }
    }
}

// ####################################################################################
// ################################### SAVE ###########################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EntitySave {
    position: Isometry2<f32>,
    linvel: Vector2<f32>,
    angvel: f32,

    hull: f32,
    armor_cells: ArmorCells,
    // TODO: Buffs
    // TODO: Inventory
    // TODO: Turret
}
impl EntitySave {
    pub fn new_stationary(data: &'static EntityData, position: Isometry2<f32>) -> Self {
        Self::new(data, position, Vector2::zeros(), 0.0)
    }

    pub fn new(
        data: &'static EntityData,
        position: Isometry2<f32>,
        linvel: Vector2<f32>,
        angvel: f32,
    ) -> Self {
        Self {
            position,
            linvel,
            angvel,
            hull: data.hull_max,
            armor_cells: data.armor_cells.clone(),
        }
    }

    pub fn from_entity(entity: &Entity, battlescape: &Battlescape) -> Self {
        let body = battlescape.physics.body(entity.rb);
        Self {
            position: *body.position(),
            linvel: *body.linvel(),
            angvel: body.angvel(),
            hull: entity.hull,
            armor_cells: entity.armor_cells.clone(),
        }
    }
}

// ####################################################################################
// ################################### TEST ###########################################
// ####################################################################################

// Just to see what data looks like.
#[test]
fn test_serialize_data() {
    println!(
        "{}\n",
        serde_json::to_string_pretty(&EntityDataJson {
            hull: 456.0,
            armor_max: 123.0,
            armor_cells_size: Vector2::new(3, 3),
            armor_cells: (0u8..3 * 3).into_iter().collect(),
            shape: HullShapeJson::Polygon {
                vertices: vec![0.0, -1.0, 1.0, 1.0, -1.0, 1.0]
            },
            mass_radius: 2.0,
            density: 3.0,
            groups: group::GROUPS_SHIP,
            mobility: Mobility {
                linear_acceleration: 1.0,
                angular_acceleration: 2.0,
                max_linear_velocity: 3.0,
                max_angular_velocity: 4.0
            },
            on_new: vec![EntityEvent::Ship, EntityEvent::Seek]
        })
        .unwrap()
    );

    println!(
        "{}\n",
        serde_json::to_string_pretty(&EntitySave {
            position: Default::default(),
            linvel: Default::default(),
            angvel: Default::default(),
            hull: 456.0,
            armor_cells: (0u8..3 * 3).into_iter().collect(),
        })
        .unwrap()
    );
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
