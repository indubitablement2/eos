use super::*;

pub struct InstanceConnection {
    pub connection: Connection,

    pub battlescapes: Vec<BattlescapeId>,
}
impl InstanceConnection {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            battlescapes: Default::default(),
        }
    }

    pub fn create_battlescape(&mut self, id: BattlescapeId) {
        self.battlescapes.push(id);
        self.connection
            .send(CentralInstancePacket::CreateBattlescape { id });
    }
}
