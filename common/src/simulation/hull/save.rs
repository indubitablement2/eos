use super::*;

#[derive(Serialize, Deserialize, Default)]
pub enum HullSave {
    #[default]
    V0,
    // V1 {}
}
impl HullSave {
    pub fn to_hull_builder(mut self) -> HullBuilder {
        loop {
            match self._to_hull_builder() {
                Ok(builder) => return builder,
                Err(save) => self = save,
            }
        }
    }

    pub fn from_hull(hull: &Hull) -> Self {
        // TODO: Saving hull!
        Self::V0
    }

    fn _to_hull_builder(self) -> Result<HullBuilder, Self> {
        Err(match self {
            HullSave::V0 => return Ok(HullBuilder::default()),
            // HullSave::V1 {} => HullBuilder::default(),
        })
    }
}
