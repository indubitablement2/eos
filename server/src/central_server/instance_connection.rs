use super::*;

pub struct InstanceConnection {
    pub connection: Connection,

    pub work: (),
}
impl InstanceConnection {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            work: (),
        }
    }
}
