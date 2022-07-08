use super::*;

pub mod know_fleets;

pub use self::know_fleets::*;

pub struct ClientConnection<C>
where
    C: Connection,
{
    pub connection: C,
    pub auth: Auth,
    /// Exclude the client's fleet.
    pub know_fleets: KnowFleets,
    pub last_position: Vec2,
    pub last_detector_radius: f32,
    pub last_in_system: Option<SystemId>,
    pub ghost: bool,
}

impl<C> ClientConnection<C>
where
    C: Connection,
{
    pub fn new(connection: C, auth: Auth) -> Self {
        Self {
            connection,
            auth,
            know_fleets: Default::default(),
            last_position: Default::default(),
            last_detector_radius: 1.0,
            ghost: true,
            last_in_system: None,
        }
    }
}
