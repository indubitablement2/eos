use std::iter::once;

use ahash::AHashSet;
use common::compressed_vec2::CVec2;

use crate::serverW::*;

/// ### This should be called after fleets are done changing.
///
/// Update client's detected fleet.
///
/// Send changed fleets infos.
///
/// Send detected fleets position.
///
/// Flush tcp buffer.
pub fn send_detected_entities(s: &mut Server) {
    let (detected_fleets, connection) =
        query_ptr!(s.clients, Client::detected_fleets, Client::connection);

    for i in 0..s.clients.len() {
        let (connection, detected_fleets) =
            unsafe { (&*connection.add(i), &mut *detected_fleets.add(i)) };

        let client_fleet_id = connection.client_id().to_fleet_id();
        let client_fleet_index = s.fleets.get_index(client_fleet_id).unwrap();
        let (client_position, client_last_change) = query!(
            s.fleets,
            client_fleet_index,
            Fleet::position,
            Fleet::last_change
        );

        let interval = s
            .server_configs
            .performance_configs
            .client_detected_entity_update_interval;

        let changed = if connection.client_id().0 % interval == 0 {
            // Update what this client has detected.

            let client_fleet_id = connection.client_id().to_fleet_id();
            let client_fleet_index = s.fleets.get_index(client_fleet_id).unwrap();
            let (fleet_detector_radius, fleet_in_system) = query!(
                s.fleets,
                client_fleet_index,
                Fleet::detector_radius,
                Fleet::in_system
            );

            let mut know = AHashSet::with_capacity(detected_fleets.len());
            know.extend(detected_fleets.iter().copied());
            detected_fleets.clear();
            let mut no_know = Vec::new();

            // Detect nearby fleets.
            let collider = Collider::new(
                *client_position,
                *fleet_detector_radius,
                fleet_in_system
                    .to_owned()
                    .map(|system_id| system_id.0)
                    .unwrap_or(u32::MAX),
            );
            s.fleets_detection_acceleration_structure
                .intersect_collider(collider, |other| {
                    // All newly detected fleets.
                    if !know.contains(&other.id) {
                        no_know.push(other.id);
                    }

                    // Do not add the client's fleet.
                    if other.id != client_fleet_id {
                        detected_fleets.push(other.id);
                    }

                    false
                });
            detected_fleets.sort_unstable();

            // know = fleets we previously had detected (and may still have).
            // no_know = fleets we gained detection and (100% do not know).
            // detected_fleets = all currently detected fleets.

            // Check if knows fleet have changed.
            no_know.extend(know.into_iter().filter_map(|fleet_id| {
                let fleet_index = s.fleets.get_index(fleet_id).unwrap();
                let last_change = query!(s.fleets, fleet_index, Fleet::last_change).0;

                if *last_change + interval > s.time.tick {
                    Some(fleet_id)
                } else {
                    None
                }
            }));

            no_know
        } else {
            // Check if detected fleet have changed.
            detected_fleets
                .iter()
                .filter_map(|&fleet_id| {
                    let fleet_index = s.fleets.get_index(fleet_id).unwrap();
                    let last_change = query!(s.fleets, fleet_index, Fleet::last_change).0;

                    if *last_change == s.time.tick {
                        Some(fleet_id)
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Send detected fleets & client's fleet infos the client does not know.
        connection.send_packet_reliable(
            ServerPacket::DetectedFleetsInfos(DetectedFleetsInfos {
                tick: s.time.tick,
                infos: changed
                    .into_iter()
                    .chain(once(client_fleet_id).filter(|_| s.time.tick == *client_last_change))
                    .map(|fleet_id| {
                        let fleet_index = s.fleets.get_index(fleet_id).unwrap();
                        let (name, orbit, composition) = query!(
                            s.fleets,
                            fleet_index,
                            Fleet::name,
                            Fleet::orbit,
                            Fleet::composition
                        );

                        FleetInfos {
                            fleet_id,
                            name: name.to_owned(),
                            orbit: orbit.to_owned(),
                            composition: composition.to_owned(),
                        }
                    })
                    .collect(),
            })
            .serialize(),
        );

        // Send detected fleets position.
        connection.send_packet_unreliable(
            ServerPacket::FleetsPosition(FleetsPosition {
                tick: s.time.tick,
                client_position: *client_position,
                relative_fleets_position: detected_fleets
                    .iter()
                    .map(|fleet_id| {
                        let fleet_index = s.fleets.get_index(*fleet_id).unwrap();
                        let fleet_position = query!(s.fleets, fleet_index, Fleet::position).0;
                        CVec2::from_vec2(*fleet_position, CVec2::METASCAPE_RANGE)
                    })
                    .collect(),
            })
            .serialize(),
        );

        // Flush the tcp buffer.
        connection.flush_tcp_stream();
    }
}
