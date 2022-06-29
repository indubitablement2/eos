use super::*;

pub struct KnowFleets {
    /// If the full client's fleet infos should be sent as if it was just detected.
    /// Will be reset to false when sent.
    pub update_client: bool,
    pub order: Vec<FleetId>,
    pub order_checksum: u32,
    pub period_number: u32,
    pub recently_removed: Vec<FleetId>,
    pub recently_added: Vec<FleetId>,
}

impl KnowFleets {
    pub fn clear(&mut self) {
        self.order.clear();
        self.recently_removed.clear();
        self.recently_added.clear();
        self.order_checksum = 0;
        self.period_number = 0;
    }
}

impl Default for KnowFleets {
    fn default() -> Self {
        Self {
            update_client: true,
            order: Default::default(),
            order_checksum: 0,
            period_number: 0,
            recently_removed: Default::default(),
            recently_added: Default::default(),
        }
    }
}
