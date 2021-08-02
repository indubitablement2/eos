use crate::idx::*;
use crate::location::Location;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Complete fleet data.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FleetData {
    /// Unique id for this fleet.
    pub fleet_id: FleetId,
    /// If a player own this fleet, his unique client_id.
    pub owner_client_id: ClientId,
    pub owner_username: String,
    pub fleet_name: String,
    pub location: Location,
    pub velocity: Vec2,
    pub ships: Vec<Ship>,
    pub cargo: Vec<(Items, u32)>,
}

/// Used for networking. Sent to other player, so they can see this fleet.
pub struct SmallFleetData {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Ship {
    pub hull: Hull,
    pub modules: Vec<Items>,
    pub weapons: Vec<Items>,
    pub condition: u8,
}

#[repr(u16)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Hull {
    Test1,
    Test2,
}
impl Hull {
    pub const fn get_armor(&self) -> i32 {
        match self {
            Hull::Test1 => 666,
            _ => 0,
        }
    }
}

#[repr(u16)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Items {
    Test1,
    Test2,
}
impl Items {
    pub const fn get_damage(&self) -> i32 {
        match self {
            Items::Test1 => 123,
            _ => 0,
        }
    }
}
