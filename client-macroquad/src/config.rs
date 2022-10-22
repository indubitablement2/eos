use crate::{inputs::InputMap, time_manager::TimeManagerConfig};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub input_map: InputMap,

    pub metascape_time_manager: TimeManagerConfig,
    pub battlescape_time_manager: TimeManagerConfig,
}
