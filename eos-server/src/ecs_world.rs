use crate::ecs_resoure::*;
use crate::ecs_system::*;
use crate::global::GlobalListWrapper;
use ahash::AHashMap;
use bevy_ecs::prelude::*;
use crossbeam_channel::Sender;
use eos_common::data::FleetData;
use eos_common::{const_var::*, idx::*};
use glam::*;
use rayon::prelude::*;
use std::convert::TryInto;

/// Grid of sectors.
pub struct SpaceGrid {
    pub sectors: Vec<Sector>,
    // /// Sent by sector when they receive a client. Used to track their location.
    // pub client_change_sector_receiver: Receiver<(u32, u16)>
}

impl SpaceGrid {
    /// Update each sectors in parallele.
    pub fn update(&mut self) {
        self.sectors.par_iter_mut().for_each(|sector| {
            sector.update();
        });
    }

    /// Create a new SpaceGrid of Sector. Also return a copy of senders to every sector.
    pub fn new(global_list_wrapper: &GlobalListWrapper) -> (SpaceGrid, Vec<Sender<FleetData>>) {
        let mut sectors = Vec::with_capacity(NUM_SECTOR);

        // Temporary map that hold channel to be distributed.
        let mut t_map = AHashMap::with_capacity(NUM_SECTOR);
        for i in 0..NUM_SECTOR {
            let channel = crossbeam_channel::unbounded::<FleetData>();
            t_map.insert(i, channel);
        }

        // Save the sender map for the login loop.
        let mut senders_copy = Vec::with_capacity(NUM_SECTOR);
        for i in 0..NUM_SECTOR {
            senders_copy.push(t_map.get(&i).unwrap().0.clone());
        }

        // Filling the space_grid with sector.
        for i in 0..NUM_SECTOR {
            // Distributing channels to sector.
            let r = t_map.get(&i).unwrap().1.clone();
            let my_vec = sector_id_to_ivec(i.try_into().unwrap());
            let sr = t_map
                .get(&(ivec_to_sector_id(ivec2((my_vec.x + 1).rem_euclid(X_SECTOR), my_vec.y)) as usize))
                .unwrap()
                .0
                .clone();
            let sb = t_map
                .get(&(ivec_to_sector_id(ivec2(my_vec.x, (my_vec.y + 1).rem_euclid(Y_SECTOR))) as usize))
                .unwrap()
                .0
                .clone();
            let sl = t_map
                .get(&(ivec_to_sector_id(ivec2((my_vec.x - 1).rem_euclid(X_SECTOR), my_vec.y)) as usize))
                .unwrap()
                .0
                .clone();
            let st = t_map
                .get(&(ivec_to_sector_id(ivec2(my_vec.x, (my_vec.y - 1).rem_euclid(Y_SECTOR))) as usize))
                .unwrap()
                .0
                .clone();
            let sec_com = SectorCommunicationRes {
                receive_entity: r,
                send_entity_right: sr,
                send_entity_bot: sb,
                send_entity_left: sl,
                send_entity_top: st,
                fleet_current_sector_insert_send: global_list_wrapper
                    .global_list_channel
                    .fleet_current_sector_insert_send
                    .clone(),
            };

            // Adding sector to SpaceGrid.
            sectors.push(Sector::new(
                SectorId(i.try_into().unwrap()),
                sec_com,
                GlobalListRes(global_list_wrapper.global_list.clone()),
            ));
        }

        (SpaceGrid { sectors }, senders_copy)
    }
}

fn sector_id_to_ivec(sector_id: i32) -> IVec2 {
    ivec2(sector_id % X_SECTOR, sector_id / X_SECTOR)
}

fn ivec_to_sector_id(sector_vec: IVec2) -> i32 {
    sector_vec.x + sector_vec.y * X_SECTOR
}

/// Each sector is an ECS world.
pub struct Sector {
    /// Unique id also position within SpaceGrid.sectors.
    pub id: SectorId,
    world: World,
    schedule: Schedule,
}

impl Sector {
    /// Make a new Sector from various parameter.
    fn new(id: SectorId, sec_com: SectorCommunicationRes, global_list_res: GlobalListRes) -> Sector {
        let mut world = World::default();
        world.insert_resource(sec_com);
        world.insert_resource(SectorIdRes(id));
        world.insert_resource(global_list_res);
        world.insert_resource(SectorTimeRes {
            tick: 0,
            delta: 0.0,
            last_instant: std::time::Instant::now(),
        });

        let mut schedule = Schedule::default();
        schedule.add_stage("update", SystemStage::single_threaded());
        schedule.add_stage_before("update", "before_update", SystemStage::single_threaded());
        schedule.add_stage_after("update", "apply_update", SystemStage::single_threaded());
        schedule.add_stage_after("apply_update", "last", SystemStage::single_threaded());

        // before_update
        schedule.add_system_to_stage("before_update", get_new_fleet.system());
        schedule.add_system_to_stage("before_update", update_sector_time.system());

        // update
        schedule.add_system_to_stage("update", fleet_movement.system());

        // apply_update
        schedule.add_system_to_stage("apply_update", apply_velocity.system());

        // last
        schedule.add_system_to_stage("last", change_sector.system());

        Sector { id, world, schedule }
    }

    /// Run the schedule once on this sector's world.
    fn update(&mut self) {
        self.schedule.run(&mut self.world); // TODO: run() or run_once() ?
    }
}
