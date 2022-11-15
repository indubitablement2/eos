use super::*;

#[derive(Debug, Clone, Default)]
pub struct RenderEvents {
    pub ship_added: Vec<ShipId>,
}
