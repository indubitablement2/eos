use super::*;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub data: &'static EntityData,

    pub body: RigidBodyHandle,
    pub hulls: Box<[Result<Hull, &'static HullData>]>,

    pub mobility: Mobility,
}
impl Entity {
    // pub fn new(data: EntityDataId, physics: &mut Physics) -> Self {
    //     let body_builder = SimpleRigidBodyBuilder::dynamic();
    //     physics.add_body(body_builder, id)
    //     let (body, collider) = physics.add_body_collider(
    //         data.shape.clone(),
    //         BodyStatus::Dynamic,
    //         1.0,
    //         0.0,
    //         0.0,
    //         0.0,
    //         0.0,
    //     );

    //     Self {
    //         data,
    //         body,
    //         collider,
    //     }
    // }

    pub fn step(&mut self) {
        //
    }

    pub fn is_destroyed(&self) -> bool {
        self.hulls
            .first()
            .expect("entiy should have at least one hull")
            .is_err()
    }

    pub fn remove(&mut self, physics: &mut Physics) {
        physics.remove_body(self.body);
    }
}

impl serde::Serialize for &'static EntityData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.idx)
    }
}
impl<'de> serde::Deserialize<'de> for &'static EntityData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let idx = u16::deserialize(deserializer)?;
        Ok(&Data::data().entities[idx as usize])
    }
}

pub struct EntityData {
    pub idx: u16,

    pub body: RigidBody,
    /// missile, fighter, projectile
    pub is_tiny: bool,
    pub wish_ignore_tiny: bool,

    pub hulls: Box<[&'static HullData]>,
    pub mobility: Mobility,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Mobility {
    /// In unit/seconds.
    pub linear_acceleration: f32,
    /// In radian/seconds.
    pub angular_acceleration: f32,
    /// In unit/seconds.
    pub max_linear_velocity: f32,
    /// In radian/seconds.
    pub max_angular_velocity: f32,
}
