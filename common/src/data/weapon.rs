use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponSize {
    Light,
    Medium,
    Heavy,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponBase {
    pub name: String,
    pub size: WeaponSize,
}
