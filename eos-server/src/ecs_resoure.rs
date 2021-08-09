use crate::global::GlobalList;
use bevy_ecs::prelude::*;
use eos_common::data::*;
use eos_common::idx::*;
use flume::{Receiver, Sender};
use std::time::Instant;

/// This sector's unique Id.
pub struct SectorIdRes(pub SectorId);

/// Time related variables.
pub struct SectorTimeRes {
    pub last_instant: Instant,
    pub tick: u32,
    pub delta: f32,
}

/// Channels for fleet to/from this sectors.
pub struct SectorCommunicationRes {
    // Entry point for fleet into the sector.
    pub receive_entity: Receiver<FleetData>,
    /// Send received fleet to update global list.
    pub fleet_current_sector_insert_send: Sender<(Vec<FleetId>, SectorId)>,

    /// Send entity to right neighbour sector.
    pub send_entity_right: Sender<FleetData>,
    /// Send entity to bottom neighbour sector.
    pub send_entity_bot: Sender<FleetData>,
    /// Send entity to left neighbour sector.
    pub send_entity_left: Sender<FleetData>,
    /// Send entity to top neighbour sector.
    pub send_entity_top: Sender<FleetData>,
    // /// Send entity to be disconnected.
    // to_disconnect_sender: Sender<ClientData>,
}

/// Pointer to the GlobalList. Require unsafe code to utilise.
pub struct GlobalListRes(pub *const Box<GlobalList>);
unsafe impl Send for GlobalListRes {}
unsafe impl Sync for GlobalListRes {}

pub struct FleetInSystemRes(Vec<Vec<Entity>>);
