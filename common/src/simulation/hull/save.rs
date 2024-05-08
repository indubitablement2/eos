use super::*;

// TODO: Inventory
// TODO: Turret
#[derive(Serialize, Deserialize, Default)]
pub enum HullSave {
    #[default]
    V0,
    // V1 {}
}
impl HullSave {
    pub fn apply(self, hull: &mut Hull) {
        match self {
            HullSave::V0 => (),
        }
    }

    pub fn from_hull(hull: &Hull) -> Self {
        // TODO: Saving hull!
        Self::V0
    }
}
