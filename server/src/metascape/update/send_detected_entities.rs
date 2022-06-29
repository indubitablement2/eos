use std::iter::once;

use crate::metascape::*;
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
    let fleets_out_detection_acc = &s.fleets_out_detection_acc;
    let fleets_in_detection_acc = &s.fleets_in_detection_acc;

    let connections_connection = s.connections.container.connection.as_slice();
    let connections_know_fleets = s.connections.container.know_fleets.as_mut_slice();

    let fleets_index_map = &s.fleets.index_map;
    let fleets_position = s.fleets.container.position.as_slice();
    let fleets_inner = s.fleets.container.fleet_inner.as_slice();
    let fleets_in_system = s.fleets.container.in_system.as_slice();
    let fleets_name = s.fleets.container.name.as_slice();
    let fleets_orbit = s.fleets.container.orbit.as_slice();

    let interval = s
        .server_configs
        .performance_configs
        .client_detected_entity_update_interval;
    let step = tick() % interval;

    for (connection, know_fleets) in connections_connection.iter().zip(connections_know_fleets) {
        let client_id = connection.client_id();
        let client_fleet_id = client_id.to_fleet_id();

        let client_fleet_index =
            if let Some(client_fleet_index) = fleets_index_map.get(&client_fleet_id) {
                *client_fleet_index
            } else {
                // Client has no fleet thus can not detect nearby entities.

                // TODO: Keep notifying the client / use ghost?
                know_fleets.clear();

                connection.flush_tcp_stream();
                continue;
            };

        let client_fleet_position = fleets_position[client_fleet_index];
        let client_fleet_inner = &fleets_inner[client_fleet_index];

        let full_update = client_id.0 % interval == step;

        // Detect things around the client's fleet.
        if full_update {
            // Update what this client has detected.

            know_fleets.recently_added.clear();
            know_fleets.recently_removed.clear();
            know_fleets.period_number += 1;

            // Get the proper acceleration structure.
            let acc = if let Some(system_id) = fleets_in_system[client_fleet_index] {
                if let Some(acc) = fleets_in_detection_acc.get(&system_id) {
                    acc
                } else {
                    log::warn!("");
                    continue;
                }
            } else {
                fleets_out_detection_acc
            };

            // Create the collider we will be using.
            let collider = Circle::new(
                client_fleet_position,
                client_fleet_inner.fleet_stats().detector_radius,
            );

            // Detect nearby fleets.
            let mut detected = AHashSet::new();
            acc.intersect(&collider, |_, other| {
                if *other != client_fleet_id {
                    detected.insert(*other);
                }

                false
            });

            // Remove fleet we lost detection.
            know_fleets.order.drain_filter(|fleet_id| {
                if !detected.remove(fleet_id) {
                    // Fleet is no more detected.
                    know_fleets.recently_removed.push(*fleet_id);
                    true
                } else {
                    false
                }
            });
            // Add fleet we just detected.
            for &fleet_id in detected.iter() {
                know_fleets.order.push(fleet_id);
                know_fleets.recently_added.push(fleet_id);
            }

            // Compute the order checksum.
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(bytemuck::cast_slice(&know_fleets.order));
            know_fleets.order_checksum = hasher.finalize();
        }

        let check_change_range = if full_update {
            // New fleet will send full info anyway.
            know_fleets.order.len() - know_fleets.recently_added.len()
        } else {
            know_fleets.order.len()
        };

        // Check if known fleets have changed.
        let mut compositions_changed = Vec::new();
        let mut orbits_changed = Vec::new();
        know_fleets.order[0..check_change_range]
            .iter()
            // Add the client's fleet if we are not sending a full update.
            .chain(once(&client_fleet_id).filter(|_| !know_fleets.update_client))
            .for_each(|fleet_id| {
                if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                    // Check for fleet composition change.
                    let fleet_inner = &fleets_inner[fleet_index];
                    if fleet_inner.last_change() == tick() {
                        compositions_changed
                            .push((*fleet_id, fleet_inner.fleet_composition().to_owned()));
                    }

                    // Check for orbit change.
                    if let Some((orbit, added_tick)) = fleets_orbit[fleet_index] {
                        if added_tick == tick() {
                            orbits_changed.push((*fleet_id, orbit));
                        }
                    }
                }
            });

        // Get the full info for new fleet.
        let new_fleets: Vec<FleetInfos> = know_fleets.order[check_change_range..]
            .iter()
            // Add the client's fleet if we were requested to send a full update.
            .chain(once(&client_fleet_id).filter(|_| know_fleets.update_client))
            .filter_map(|fleet_id| {
                if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                    Some(FleetInfos {
                        fleet_id: *fleet_id,
                        name: fleets_name[fleet_index].to_owned(),
                        orbit: fleets_orbit[fleet_index].map(|(orbit, _)| orbit),
                        fleet_composition: fleets_inner[fleet_index].fleet_composition().to_owned(),
                    })
                } else {
                    None
                }
            })
            .collect();

        know_fleets.update_client = false;

        // Send detected fleets infos the client does not know.
        if !new_fleets.is_empty() || !compositions_changed.is_empty() || !orbits_changed.is_empty()
        {
            connection.send_packet_reliable(
                ServerPacket::FleetsInfos(FleetsInfos {
                    tick: tick(),
                    new_fleets,
                    compositions_changed,
                    orbits_changed,
                })
                .serialize(),
            );
        }

        // Send needed detected fleets position.
        let mut sent_bitfield = vec![0; know_fleets.order.len() / 8 + 1];
        // TODO: If we have too many fleets, only send some.
        let relative_fleets_position = know_fleets
            .order
            .iter()
            .enumerate()
            .filter_map(|(i, fleet_id)| {
                if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                    if fleets_orbit[fleet_index].is_none() {
                        // Set the sent bit to true.
                        let bitfield_index = i / 8;
                        let bit_index = i % 8;
                        sent_bitfield[bitfield_index] |= 1 << bit_index;

                        Some(CVec2::from_vec2(
                            fleets_position[fleet_index] - client_fleet_position,
                        ))
                    } else {
                        // No need to send orbiting fleet position.
                        None
                    }
                } else {
                    // Fleet does not exist anymore.
                    None
                }
            })
            .collect();
        connection.send_packet_unreliable(
            ServerPacket::FleetsPosition(FleetsPosition {
                tick: tick(),
                ghost: false,
                origin: client_fleet_position,
                relative_fleets_position,
                sent_bitfield,
                order_add: know_fleets.recently_added.to_owned(),
                order_remove: know_fleets.recently_removed.to_owned(),
                order_checksum: know_fleets.order_checksum,
                period_number: know_fleets.period_number,
            })
            .serialize(),
        );

        // Flush the tcp buffer.
        connection.flush_tcp_stream();
    }
}
