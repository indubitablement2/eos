use super::*;

pub struct KnowFleets {
    /// If the full client's fleet infos should be sent as if it was just detected.
    /// Will be reset to false when sent.
    pub update_client: bool,
    pub order: Vec<FleetId>,
    pub order_checksum: u32,
    /// Order changes that were not acknowleged.
    pub non_ack_change: Vec<MetascapeStateOrderChange>,
}

impl Default for KnowFleets {
    fn default() -> Self {
        Self {
            update_client: true,
            order: Default::default(),
            order_checksum: 0,
            non_ack_change: Default::default(),
        }
    }
}
