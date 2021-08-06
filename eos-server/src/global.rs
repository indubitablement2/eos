use ahash::AHashMap;
use flume::{unbounded, Receiver, Sender};
use eos_common::connection_manager::Connection;
use eos_common::idx::*;
use parking_lot::RwLock;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub struct GlobalListWrapper {
    /// Many important information. Only updated on main.
    pub global_list: Arc<RwLock<GlobalList>>,
    /// Used to modify GlobalList.
    pub global_list_channel: GlobalListChannel,

    /// Used to asign a unique id to a new fleet.
    pub fleet_id_counter: Arc<AtomicU64>,
}

impl GlobalListWrapper {
    pub fn new(fleet_id_last_counter: u64) -> GlobalListWrapper {
        let (global_list, global_list_channel) = GlobalList::new();

        GlobalListWrapper {
            global_list,
            global_list_channel,
            fleet_id_counter: Arc::new(AtomicU64::new(fleet_id_last_counter)),
        }
    }

    /// Update the lists.
    pub fn update(&mut self) {
        // * Lock.
        let mut global_list_write = self.global_list.write();

        // * Fleet current sector update.
        while let Ok((vec_fleet_ids, sector_id)) = self.global_list_channel.fleet_current_sector_insert_receive.try_recv() {
            vec_fleet_ids.into_iter().for_each(|fleet_id| {
                global_list_write.fleet_current_sector.insert(fleet_id, sector_id);
                trace!("Fleet {:?} entered {:?}.", fleet_id, sector_id);
            });
        }

        // * Fleet current sector remove.
        while let Ok(vec_fleet_ids) = self.global_list_channel.fleet_current_sector_remove_receive.try_recv() {
            vec_fleet_ids.into_iter().for_each(|fleet_id| {
                global_list_write.fleet_current_sector.remove(&fleet_id);
                trace!("Fleet {:?} removed.", fleet_id);
            });
        }
    }
}

/// Channel to/from global list.
pub struct GlobalListChannel {
    /// Sectors send FleetId with its SectorId when a fleet enter.
    pub fleet_current_sector_insert_send: Sender<(Vec<FleetId>, SectorId)>,
    /// Received from sector when a fleet enters.
    fleet_current_sector_insert_receive: Receiver<(Vec<FleetId>, SectorId)>,
    /// Sectors send FleetId when a fleet leave the SpaceGrid altogether.
    pub fleet_current_sector_remove_send: Sender<Vec<FleetId>>,
    /// Received from sector when a fleet leave the SpaceGrid altogether.
    fleet_current_sector_remove_receive: Receiver<Vec<FleetId>>,
}

pub struct GlobalList {
    /// Client currently connected.
    pub connected_client: AHashMap<ClientId, Connection>,
    /// The current sector of every fleet.
    /// It could happen that a client's fleet remain in a sector while client is not connected.
    /// For example if client leave during a fight.
    pub fleet_current_sector: AHashMap<FleetId, SectorId>,
}

impl GlobalList {
    fn new() -> (Arc<RwLock<GlobalList>>, GlobalListChannel) {
        let (fleet_current_sector_insert_send, fleet_current_sector_insert_receive) = unbounded();
        let (fleet_current_sector_remove_send, fleet_current_sector_remove_receive) = unbounded();

        (
            Arc::new(RwLock::new(GlobalList {
                connected_client: AHashMap::with_capacity(1024),
                fleet_current_sector: AHashMap::with_capacity(1024),
            })),
            GlobalListChannel {
                fleet_current_sector_insert_send,
                fleet_current_sector_insert_receive,
                fleet_current_sector_remove_send,
                fleet_current_sector_remove_receive,
            },
        )
    }
}
