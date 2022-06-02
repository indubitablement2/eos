use crate::metascape::*;
use ahash::AHashMap;
use std::iter::once;
use utils::compressed_vec2::CVec2;

/// ### This should be called after entities are done changing.
///
/// Update client's detected fleet.
///
/// Send changed fleets infos.
///
/// Send detected fleets position.
///
/// Flush tcp buffer.
pub fn send_detected_entities(s: &mut Metascape) {
    let (know_fleets, connection) = query_ptr!(s.clients, Client::know_fleets, Client::connection);

    for i in 0..s.clients.len() {
        let (connection, know_fleets) = unsafe { (&*connection.add(i), &mut *know_fleets.add(i)) };

        let client_fleet_id = connection.client_id().to_fleet_id();
        let client_fleet_index = s.fleets.get_index(client_fleet_id).unwrap();
        let (client_position, client_fleet_inner) = query!(
            s.fleets,
            client_fleet_index,
            Fleet::position,
            Fleet::fleet_inner
        );

        let client_last_change = client_fleet_inner.last_change();

        let interval = s
            .server_configs
            .performance_configs
            .client_detected_entity_update_interval;

        // Get the fleets that changed. Excluding the client's fleet.
        let changed = if connection.client_id().0 % interval == 0 {
            // Update what this client has detected.

            let client_in_system = query!(s.fleets, client_fleet_index, Fleet::in_system).0;

            let client_detector_radius = client_fleet_inner.fleet_stats().detector_radius;

            let mut fleets_to_forget = AHashMap::with_capacity(know_fleets.fleets.len());
            fleets_to_forget.extend(know_fleets.fleets.iter().copied());
            know_fleets.fleets.clear();
            know_fleets.reuse_small_id();
            let mut no_know = Vec::new();

            // Detect nearby fleets.
            let collider = Collider::new(
                *client_position,
                client_detector_radius,
                client_in_system
                    .to_owned()
                    .map(|system_id| system_id.0)
                    .unwrap_or(u32::MAX),
            );
            s.fleets_detection_acceleration_structure
                .intersect_collider(collider, |other| {
                    // Do not add the client's fleet.
                    if other.id != client_fleet_id {
                        if let Some(small_id) = fleets_to_forget.remove(&other.id) {
                            // We already knew about this fleet.
                            know_fleets.fleets.push((other.id, small_id));
                        } else {
                            // We detected a new fleet.
                            no_know.push(other.id);
                        }
                    }

                    false
                });

            // fleets_to_forget = fleets we previously had detected and do not anymore.
            // no_know = fleets we just gained detection and 100% do not know.
            // know_fleets.fleets = fleets we already knew and are still detected.

            // Recycle small_id and send fleets to forget to client.
            let mut to_forget = Vec::with_capacity(fleets_to_forget.len());
            for &small_id in fleets_to_forget.values() {
                to_forget.push(small_id);
                know_fleets.recycle_small_id(small_id);
            }
            connection.send_packet_reliable(
                ServerPacket::FleetsForget(FleetsForget {
                    tick: time().tick,
                    to_forget,
                })
                .serialize(),
            );

            let mut changed = Vec::with_capacity(no_know.len() + know_fleets.fleets.len());

            // Check if already know fleets have changed.
            for (fleet_id, small_id) in know_fleets.fleets.iter() {
                // TODO: don't unwrap for future proof sake
                let fleet_index = s.fleets.get_index(*fleet_id).unwrap();
                let last_change = query!(s.fleets, fleet_index, Fleet::fleet_inner)
                    .0
                    .last_change();

                // TODO: make sure this does not send duplicates.
                if last_change + interval > time().tick {
                    changed.push((*fleet_id, *small_id));
                }
            }

            // Assign a small_id to newly detected fleets.
            for fleet_id in no_know {
                let small_id = know_fleets.get_new_small_id();
                know_fleets.fleets.push((fleet_id, small_id));
                changed.push((fleet_id, small_id));
            }

            changed
        } else {
            // Check if known fleets have changed.
            know_fleets
                .fleets
                .iter()
                .filter_map(|&(fleet_id, small_id)| {
                    let fleet_index = s.fleets.get_index(fleet_id).unwrap();
                    let last_change = query!(s.fleets, fleet_index, Fleet::fleet_inner)
                        .0
                        .last_change();

                    if last_change == time().tick {
                        Some((fleet_id, small_id))
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Send detected fleets & client's fleet infos the client does not know.
        connection.send_packet_reliable(
            ServerPacket::FleetsInfos(FleetsInfos {
                tick: time().tick,
                infos: changed
                    .into_iter()
                    .chain(once((client_fleet_id, 0)).filter(|_| time().tick == client_last_change))
                    .filter_map(|(fleet_id, small_id)| {
                        if let Some(fleet_index) = s.fleets.get_index(fleet_id) {
                            let (name, orbit, fleet_inner) = query!(
                                s.fleets,
                                fleet_index,
                                Fleet::name,
                                Fleet::orbit,
                                Fleet::fleet_inner
                            );

                            Some(FleetInfos {
                                fleet_id,
                                name: name.to_owned(),
                                orbit: orbit.to_owned(),
                                fleet_composition: fleet_inner.fleet_composition().to_owned(),
                                small_id,
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            })
            .serialize(),
        );

        // Send detected fleets position.
        connection.send_packet_unreliable(
            ServerPacket::FleetsPosition(FleetsPosition {
                tick: time().tick,
                client_position: *client_position,
                relative_fleets_position: know_fleets
                    .fleets
                    .iter()
                    .filter_map(|(fleet_id, small_id)| {
                        if let Some(fleet_index) = s.fleets.get_index(*fleet_id) {
                            let fleet_position = query!(s.fleets, fleet_index, Fleet::position).0;
                            Some((
                                *small_id,
                                CVec2::from_vec2(*fleet_position, common::METASCAPE_RANGE),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
            })
            .serialize(),
        );

        // Flush the tcp buffer.
        connection.flush_tcp_stream();
    }
}
