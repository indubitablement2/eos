use self::know_fleets::KnowFleets;
use super::*;

pub mod know_fleets;

#[derive(Soa)]
pub struct Connection {
    pub connection: common::net::connection::Connection,
    /// Exclude the client's fleet.
    pub know_fleets: KnowFleets,
}

pub struct ConnectionBuilder {
    connection: common::net::connection::Connection,
}
impl ConnectionBuilder {
    pub fn new(connection: common::net::connection::Connection) -> Self {
        Self { connection }
    }

    pub fn build(self) -> Connection {
        Connection {
            connection: self.connection,
            know_fleets: KnowFleets::default(),
        }
    }
}
