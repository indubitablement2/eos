pub mod know_fleets;

pub use self::know_fleets::*;
use super::*;
use common::net::connection::*;

#[derive(Fields, Columns, Components)]
pub struct Client {
    pub connection: Connection,
    /// Exclude the client's fleet.
    pub know_fleets: KnowFleets,

}

pub struct ClientBuilder {
    connection: Connection,
}
impl ClientBuilder {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn build(self) -> Client {
        Client {
            connection: self.connection,
            know_fleets: KnowFleets::default(),
        }
    }
}
