use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeClient {
    pub client_inputs: ClientInputs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientInputs {
    pub wish_linvel: WishLinVel,
    pub wish_angvel: WishAngVel,
}
impl ClientInputs {
    /// TODO: Remove NaN, infinite and invalide inputs.
    pub fn sanetize(&mut self) {}
}
