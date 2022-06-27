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

    for (connection, know_fleets) in connections_connection
        .iter()
        .zip(connections_know_fleets)
    {
        let client_id = connection.client_id();
        let client_fleet_id = client_id.to_fleet_id();
        
        let client_fleet_index = if let Some(client_fleet_index) = fleets_index_map.get(&client_fleet_id) {
            *client_fleet_index
        } else {
            // Client has no fleet thus can not detect nearby entities.

            // Clear what the client know.
            let to_forget =
                Vec::from_iter(know_fleets.fleets.drain(..).map(|(_, small_id)| small_id));
            if !to_forget.is_empty() {
                // Recycle small idx.
                to_forget
                    .iter()
                    .for_each(|small_id| know_fleets.recycle_small_id(*small_id));

                connection.send_packet_reliable(
                    ServerPacket::FleetsForget(FleetsForget {
                        tick: tick(),
                        to_forget,
                    })
                    .serialize(),
                );
            }

            connection.flush_tcp_stream();
            continue;
        };

        let client_fleet_position = fleets_position[client_fleet_index];
        let client_fleet_inner = &fleets_inner[client_fleet_index];

        let mut to_forget = Vec::new();
        let mut new_fleets = Vec::new();

        // Detect things around the client's fleet.
        if client_id.0 % interval == step {
            // Update what this client has detected.

            let mut rest = AHashMap::new();
            rest.extend(know_fleets.fleets.drain(..));

            // Detect nearby fleets.
            let acc = if let Some(system_id) = fleets_in_system[client_fleet_index] {
                fleets_in_detection_acc.get(&system_id)
            } else {
                Some(fleets_out_detection_acc)
            };
            if let Some(acc) = acc {
                let collider = Circle::new(
                    client_fleet_position,
                    client_fleet_inner.fleet_stats().detector_radius,
                );

                acc.intersect(&collider, |_, other| {
                    if let Some(small_id) = rest.remove(other) {
                        // We already knew about this fleet.
                        know_fleets.fleets.push((*other, small_id));
                    } else {
                        // We detected a new fleet.
                        if let Some(&fleet_index) = fleets_index_map.get(other) {
                            new_fleets.push(FleetInfos {
                                fleet_id: *other,
                                small_id: know_fleets.get_new_small_id(),
                                name: fleets_name[fleet_index].to_owned(),
                                orbit: fleets_orbit[fleet_index].map(|(orbit, _)| orbit).to_owned(),
                                fleet_composition: fleets_inner[fleet_index]
                                    .fleet_composition()
                                    .to_owned(),
                            });
                        }
                    }

                    false
                });
            };

            // rest = fleets we previously had detected and do not anymore.
            // new_fleets = fleets we just gained detection and 100% do not know.

            to_forget.extend(rest.values().copied());
        }

        let mut compositions_changed = Vec::new();
        let mut orbits_changed = Vec::new();

        // Check if already known fleets have changed.
        know_fleets.fleets.drain_filter(|(fleet_id, small_id)| {
            if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                // Check for fleet composition change.
                let fleet_inner = &fleets_inner[fleet_index];
                if fleet_inner.last_change() == tick() {
                    compositions_changed
                        .push((*small_id, fleet_inner.fleet_composition().to_owned()));
                }

                // Check for orbit change.
                if let Some((orbit, changed_tick)) = fleets_orbit[fleet_index] {
                    if changed_tick == tick() {
                        orbits_changed.push((*small_id, orbit));
                    }
                }

                false
            } else {
                // Fleet does not exist anymore.
                to_forget.push(*small_id);
                true
            }
        });

        // Add new fleets to known fleets.
        know_fleets.fleets.extend(
            new_fleets
                .iter()
                .map(|fleet_info| (fleet_info.fleet_id, fleet_info.small_id)),
        );

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

        // Recycle small_id.
        know_fleets.reuse_small_id();
        to_forget
            .iter()
            .for_each(|&small_id| know_fleets.recycle_small_id(small_id));

        // Send fleets to forget to client.
        if !to_forget.is_empty() {
            connection.send_packet_reliable(
                ServerPacket::FleetsForget(FleetsForget {
                    tick: tick(),
                    to_forget,
                })
                .serialize(),
            );
        }

        // Send needed detected fleets position.
        connection.send_packet_unreliable(
            ServerPacket::FleetsPosition(FleetsPosition {
                tick: tick(),
                client_fleet_position,
                relative_fleets_position: know_fleets
                    .fleets
                    .iter()
                    .filter_map(|(fleet_id, small_id)| {
                        if *fleet_id != client_fleet_id {
                            if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                                if fleets_orbit[fleet_index].is_none()  {
                                    Some((
                                        *small_id,
                                        CVec2::from_vec2(fleets_position[fleet_index] - client_fleet_position),
                                    ))
                                } else {
                                    // No need to send orbiting fleet position.
                                    None
                                }
                            } else {
                                // Fleet does not exist anymore.
                                None
                            }
                        } else {
                            // Client's fleet position is already sent.
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
