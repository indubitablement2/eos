use super::*;

pub type Childs = SmallVec<[HullId; 4]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct HullId(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Hull {
    pub hull_data_id: HullDataId,
    pub current_mobility: Mobility,
    pub current_defence: Defence,
    pub rb: RigidBodyHandle,
    /// Other hulls that are our child.
    pub childs: Childs,
    pub parent: Option<HullId>,
}
impl Hull {
    pub fn new(
        hull_builder: HullBuilder,
        rb: RigidBodyHandle,
        childs: Childs,
        parent: Option<HullId>,
    ) -> Self {
        let hull_data = hull_data(hull_builder.hull_data_id);

        Self {
            hull_data_id: hull_builder.hull_data_id,
            current_mobility: hull_data.mobility,
            current_defence: hull_data.defence,
            rb,
            childs,
            parent,
        }
    }
}

pub struct HullBuilder {
    pub hull_data_id: HullDataId,
    pub pos: na::Isometry2<f32>,
    pub linvel: na::Vector2<f32>,
    pub angvel: f32,
    pub team: u32,
}
impl HullBuilder {
    pub fn new(hull_data_id: HullDataId, pos: na::Isometry2<f32>, team: u32) -> Self {
        Self {
            hull_data_id,
            pos,
            linvel: Default::default(),
            angvel: Default::default(),
            team,
        }
    }

    pub fn with_linvel(mut self, linvel: na::Vector2<f32>) -> Self {
        self.linvel = linvel;
        self
    }

    pub fn with_angvel(mut self, angvel: f32) -> Self {
        self.angvel = angvel;
        self
    }
}
