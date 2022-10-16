use super::*;

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Defence {
    pub hull: i32,
    pub armor: i32,
}
