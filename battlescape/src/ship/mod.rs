mod mobility;

use super::*;
pub use mobility::*;

#[derive(Serialize, Deserialize)]
pub struct Ship {
    pub mobility: Mobility,
}
