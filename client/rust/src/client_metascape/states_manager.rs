use crate::time_manager::TimeManager;
use ahash::AHashMap;
use common::fleet::FleetComposition;
use common::fleet::FleetStats;
use common::idx::*;
use common::net::*;
use glam::Vec2;

#[derive(Debug, Clone)]
pub struct FleetState {
    /// The tick we first learned about this entity.
    pub discovered_tick: u32,
    pub previous_tick: u32,
    pub current_tick: u32,
    pub previous_position: Vec2,
    pub current_position: Vec2,
    pub fleet_infos: Option<FleetInfos>,
    pub fleet_stats: FleetStats,
}
impl FleetState {
    pub fn new(discovered_tick: u32) -> Self {
        Self {
            discovered_tick,
            previous_tick: 0,
            current_tick: 0,
            previous_position: Vec2::ZERO,
            current_position: Vec2::ZERO,
            fleet_infos: None,
            fleet_stats: Default::default(),
        }
    }

    pub fn get_interpolated_pos(&self, time_manager: &TimeManager, orbit_time: f32) -> Vec2 {
        let interpolation = time_manager.compute_interpolation(self.previous_tick, self.current_tick);
        self.previous_position.lerp(self.current_position, interpolation)
    }

    fn update_position(&mut self, new_tick: u32, new_position: Vec2) {
        if new_tick > self.current_tick {
            if self.previous_tick == 0 {
                // This is a special case to handle new fleet which have pos(0, 0) as default.
                // We don't want to set previous_position to (0, 0) and interpolate from there.
                self.previous_tick = new_tick;
                self.previous_position = new_position;
            } else {
                self.previous_tick = self.current_tick;
                self.previous_position = self.current_position;
            }
            self.current_tick = new_tick;
            self.current_position = new_position;
        } else if new_tick > self.previous_tick {
            self.previous_tick = new_tick;
            self.previous_position = new_position;
        } else {
            debug!(
                "Received useless state. received state tick: {} current tick: {}. Ignoring...",
                new_tick, self.current_tick
            );
        }
    }

    fn update_fleet_infos(&mut self, fleet_infos_update_tick: u32, fleet_infos: FleetInfos) {
        self.fleet_stats = fleet_infos.fleet_composition.compute_stats();
        self.fleet_infos = Some(fleet_infos);
    }

    fn update_fleet_composition(&mut self, fleet_composition: FleetComposition) {
        if let Some(fleet_infos) = &mut self.fleet_infos {
            self.fleet_stats = fleet_composition.compute_stats();
            fleet_infos.fleet_composition = fleet_composition
        } else {
            log::error!("Got fleet composition update without having the full infos.");
        }
    }
}

pub struct StatesManager {
    /// Rought position of where the client is.
    pub client_position: Vec2,
    pub client_fleet_id: FleetId,

    pub order: Vec<FleetId>,
    pub fleets_state: AHashMap<FleetId, FleetState>,

    pub metascape_state_buffer: Vec<MetascapeState>,
    pub fleets_infos_buffer: Vec<FleetsInfos>,

    /// The max tick we have the order.
    /// Used for fleets infos to not go faster than state.
    max_tick: u32,
}
impl StatesManager {
    pub fn new(client_fleet_id: FleetId) -> Self {
        Self {
            client_position: Default::default(),
            client_fleet_id,
            order: Default::default(),
            fleets_state: Default::default(),
            metascape_state_buffer: Default::default(),
            fleets_infos_buffer: Default::default(),
            max_tick: Default::default(),
        }
    }

    pub fn get_client_fleet(&self) -> Option<&FleetState> {
        self.fleets_state.get(&self.client_fleet_id)
    }

    pub fn update(&mut self, tick: u32) -> Option<DisconnectedReason> {
        // Consume fleets positions.
        let mut to_consume: Vec<MetascapeState> = self
            .metascape_state_buffer
            .drain_filter(|metascape_state| metascape_state.tick <= tick)
            .collect();
        // We will consume this in order to avoid overwritting more recent data.
        to_consume.sort_by(|a, b| a.tick.cmp(&b.tick));
        for metascape_state in to_consume {
            // Handle order change.
            for mut change in metascape_state.non_ack_change {
                if change.change_tick > self.max_tick {
                    // Remove fleets.
                    for fleet_id in change.order_remove.iter() {
                        self.fleets_state.remove(fleet_id);
                    }
                    self.order.retain(|fleet_id| !change.order_remove.contains(&fleet_id));

                    // Add new fleets.
                    for fleet_id in change.order_add.iter() {
                        self.fleets_state.insert(*fleet_id, FleetState::new(tick));
                    }
                    self.order.append(&mut change.order_add);
                }
            }
            self.max_tick = self.max_tick.max(metascape_state.tick);

            // Checksum.
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(bytemuck::cast_slice(&self.order));
            if hasher.finalize() != metascape_state.order_checksum {
                log::error!("Order checksum do not match.");
                return Some(DisconnectedReason::OrderOutOfSync);
            }

            // Update our fleet position.
            if !metascape_state.ghost {
                self.fleets_state
                    .entry(self.client_fleet_id)
                    .or_insert(FleetState::new(tick))
                    .update_position(metascape_state.tick, metascape_state.origin);
            } else {
                // Remove our fleet.
                self.fleets_state.remove(&self.client_fleet_id);
            }

            self.client_position = metascape_state.origin;

            // Update other fleets position.
            self.order[metascape_state.sent_order_start as usize..]
                .iter()
                .zip(metascape_state.relative_fleets_position)
                .for_each(|(fleet_id, position)| {
                    if let Some(fleet_state) = self.fleets_state.get_mut(fleet_id) {
                        let position = position.to_vec2() + metascape_state.origin;
                        fleet_state.update_position(metascape_state.tick, position);
                    } else {
                        log::error!("Received position for {:?}, but it is not found.", fleet_id);
                    }
                });
        }

        // Consume fleets infos.
        self.fleets_infos_buffer
            .drain_filter(|fleets_infos| fleets_infos.tick <= self.max_tick)
            .for_each(|fleets_infos| {
                // Add new fleets.
                for fleet_infos in fleets_infos.new_fleets {
                    let fleet_id = fleet_infos.fleet_id;
                    if let Some(fleet_state) = self.fleets_state.get_mut(&fleet_id) {
                        fleet_state.update_fleet_infos(fleets_infos.tick, fleet_infos);
                    } else {
                        // We received this way too late. Fleet was removed.
                        log::debug!("Received out of date new fleet infos. Ignoring...");
                    }
                }

                // Update changed composition.
                for (fleet_id, fleet_composition) in fleets_infos.compositions_changed {
                    if let Some(fleets_state) = self.fleets_state.get_mut(&fleet_id) {
                        fleets_state.update_fleet_composition(fleet_composition);
                    } else {
                        // This could be out of date data or a bug.
                        log::debug!("Received composition change for {:?}, but it is not found.", fleet_id);
                    }
                }
            });

        None
    }
}
