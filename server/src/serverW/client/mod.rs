use common::net::connection::*;
use super::*;

#[derive(Fields, Columns, Components)]
pub struct Client {
    pub connection: Connection,
}

pub struct ClientBuilder {
    connection: Connection,
}
impl ClientBuilder {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
        }
    }

    pub fn build(self) -> Client {
        Client { connection: self.connection }
    }
}