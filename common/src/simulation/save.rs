use super::*;

#[derive(Default)]
pub struct SimulationBuilder {
    // TODO: Debris
    // TODO: items
    // TODO: planets state
}

#[derive(Serialize, Deserialize, Default)]
pub enum SimulationSave {
    #[default]
    V0,
    // V1 {}
}
impl SimulationSave {
    pub fn to_builder(mut self) -> SimulationBuilder {
        loop {
            match self._to_builder() {
                Ok(builder) => return builder,
                Err(save) => self = save,
            }
        }
    }

    pub fn from_sim(sim: &Simulation) -> Self {
        // TODO: Saving simulation!
        Self::V0
    }

    fn _to_builder(self) -> Result<SimulationBuilder, Self> {
        Err(match self {
            Self::V0 => return Ok(SimulationBuilder::default()),
            // HullSave::V1 {} => HullBuilder::default(),
        })
    }
}
