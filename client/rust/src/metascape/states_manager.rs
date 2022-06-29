use ahash::AHashMap;
use common::fleet::FleetComposition;
use common::idx::*;
use common::net::packets::*;
use common::orbit::Orbit;
use glam::Vec2;

use crate::time_manager::TimeManager;

#[derive(Debug, Clone)]
pub struct FleetState {
    /// The tick we first learned about this entity.
    pub discovered_tick: u32,
    pub previous_tick: u32,
    pub current_tick: u32,
    pub previous_position: Vec2,
    pub current_position: Vec2,
    pub orbit_update_tick: u32,
    pub fleet_infos: Option<FleetInfos>,
}
impl FleetState {
    pub fn new(discovered_tick: u32) -> Self {
        Self {
            discovered_tick,
            previous_tick: 0,
            current_tick: 0,
            previous_position: Vec2::ZERO,
            current_position: Vec2::ZERO,
            orbit_update_tick: discovered_tick,
            fleet_infos: None,
        }
    }

    pub fn get_interpolated_pos(&self, time_manager: &TimeManager, orbit_time: f32) -> Vec2 {
        if let Some(orbit) = self.fleet_infos.as_ref().and_then(|fleet_infos| fleet_infos.orbit) {
            orbit.to_position(orbit_time)
        } else {
            let interpolation = time_manager.compute_interpolation(self.previous_tick, self.current_tick);
            self.previous_position.lerp(self.current_position, interpolation)
        }
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

        // Remove orbit.
        if new_tick >= self.orbit_update_tick {
            if let Some(fleet_infos) = &mut self.fleet_infos {
                fleet_infos.orbit = None;
            }
        }
    }

    fn update_fleet_infos(&mut self, fleet_infos_update_tick: u32, fleet_infos: FleetInfos) {
        self.fleet_infos = Some(fleet_infos);
        self.orbit_update_tick = fleet_infos_update_tick;
    }

    fn update_fleet_composition(&mut self, fleet_composition: FleetComposition) {
        if let Some(fleet_infos) = &mut self.fleet_infos {
            fleet_infos.fleet_composition = fleet_composition
        } else {
            log::error!("Got fleet composition update without having the full infos.");
        }
    }

    fn update_orbit(&mut self, update_tick: u32, orbit: Orbit) {
        if self.current_tick > update_tick {
            // Old/useless update.
            return;
        }
        if let Some(fleet_infos) = &mut self.fleet_infos {
            fleet_infos.orbit = Some(orbit);
            self.orbit_update_tick = update_tick;
        } else {
            log::error!("Got fleet orbit update without having the full infos.");
        }
    }
}

pub struct StatesManager {
    pub client_fleet_id: FleetId,

    pub order: Vec<FleetId>,
    pub fleets_state: AHashMap<FleetId, FleetState>,

    pub next_period: u32,
    pub fleets_position_buffer: Vec<FleetsPosition>,
    pub fleets_infos_buffer: Vec<FleetsInfos>,

    /// The max tick we have the order.
    /// Used for fleets infos to not go faster than state.
    max_tick: u32,
}
impl StatesManager {
    pub fn new(client_fleet_id: FleetId) -> Self {
        Self {
            client_fleet_id,
            order: Default::default(),
            fleets_state: Default::default(),
            next_period: Default::default(),
            fleets_position_buffer: Default::default(),
            fleets_infos_buffer: Default::default(),
            max_tick: Default::default(),
        }
    }

    pub fn get_client_fleet(&self) -> Option<&FleetState> {
        self.fleets_state.get(&self.client_fleet_id)
    }

    // TODO: Panic mode for out of sync order or missed period where we try to get back in sync with the server.
    pub fn update(&mut self, tick: u32) {
        // Consume fleets positions.
        let next_period = self.next_period;
        let mut to_consume: Vec<FleetsPosition> = self
            .fleets_position_buffer
            .drain_filter(|fleets_position| {
                fleets_position.tick <= tick && fleets_position.period_number <= next_period
            })
            .collect();
        // We will consume this in order to avoid overwritting more recent data.
        to_consume.sort_by(|a, b| a.tick.cmp(&b.tick));
        for mut fleets_position in to_consume {
            self.max_tick = self.max_tick.max(fleets_position.tick);

            // Handle new period.
            if fleets_position.period_number + 1 == self.next_period {
                self.next_period += 1;

                // Remove fleets.
                for fleet_id in fleets_position.order_remove.iter() {
                    self.fleets_state.remove(fleet_id);
                }
                self.order
                    .retain(|fleet_id| !fleets_position.order_remove.contains(&fleet_id));

                // Add new fleets.
                for fleet_id in fleets_position.order_add.iter() {
                    self.fleets_state.insert(*fleet_id, FleetState::new(tick));
                }
                self.order.append(&mut fleets_position.order_add);

                // Checksum.
                let mut hasher = crc32fast::Hasher::new();
                hasher.update(bytemuck::cast_slice(&self.order));
                if hasher.finalize() != fleets_position.order_checksum {
                    // TODO: Handle order out of sync.
                    log::error!("Order checksum do not match.")
                }
            } else if fleets_position.period_number < self.next_period.saturating_sub(1) {
                // We do not have the order to handle this anymore.
                log::debug!("Reveived an outdated fleets position. Ignoring...");
                continue;
            }

            // Update our fleet position.
            if !fleets_position.ghost {
                self.fleets_state
                    .entry(self.client_fleet_id)
                    .or_insert(FleetState::new(tick))
                    .update_position(fleets_position.tick, fleets_position.origin);
            }

            // Update other fleets position.
            let mut iter = fleets_position.relative_fleets_position.into_iter();
            self.order
                .iter()
                .enumerate()
                .filter_map(|(i, fleet_id)| {
                    let bitfield_index = i / 8;
                    let bit_index = i % 8;

                    if fleets_position.sent_bitfield[bitfield_index] & 1 << bit_index != 0 {
                        Some((
                            fleet_id,
                            iter.next()
                                .unwrap_or_else(|| {
                                    log::error!("bitfield does not match our order or sent data");
                                    utils::compressed_vec2::CVec2::default()
                                })
                                .to_vec2()
                                + fleets_position.origin,
                        ))
                    } else {
                        None
                    }
                })
                .for_each(|(fleet_id, position)| {
                    if let Some(fleet_state) = self.fleets_state.get_mut(fleet_id) {
                        fleet_state.update_position(fleets_position.tick, position);
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

                // Update changed orbit.
                for (fleet_id, orbit) in fleets_infos.orbits_changed {
                    if let Some(fleets_state) = self.fleets_state.get_mut(&fleet_id) {
                        fleets_state.update_orbit(fleets_infos.tick, orbit);
                    } else {
                        // This could be out of date data or a bug.
                        log::debug!("Received orbit change for {:?}, but it is not found.", fleet_id);
                    }
                }
            });
    }
}
