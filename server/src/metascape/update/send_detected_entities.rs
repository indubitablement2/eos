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

    let clients_connection = s.clients.container.connection.as_slice();
    let clients_know_fleets = s.clients.container.know_fleets.as_mut_slice();
    let clients_control = s.clients.container.control.as_slice();

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

    for ((connection, know_fleets), control) in clients_connection
        .iter()
        .zip(clients_know_fleets)
        .zip(clients_control)
    {
        let client_id = connection.client_id();

        let client_fleet_id = if let Some(client_fleet_id) = control 
        {
            *client_fleet_id
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

        let client_fleet_index = if let Some(client_fleet_index) = fleets_index_map.get(&client_fleet_id) {
            *client_fleet_index
        } else {
            log::error!("{:?} is controlling {:?}, but it can not be found.", client_id, client_fleet_id);
            continue;
        };
        let client_fleet_position = &fleets_position[client_fleet_index];
        let client_fleet_inner = &fleets_inner[client_fleet_index];

        // Get the fleets that changed.
        let changed = if client_id.0 % interval == step {
            // Update what this client has detected.

            let mut rest = AHashMap::new();
            rest.extend(know_fleets.fleets.drain(..));
            let mut no_know = Vec::new();

            // Detect nearby fleets.
            let acc = if let Some(system_id) = fleets_in_system[client_fleet_index] {
                fleets_in_detection_acc.get(&system_id)
            } else {
                Some(fleets_out_detection_acc)
            };
            if let Some(acc) = acc {
                let collider = Circle::new(
                    *client_fleet_position,
                    client_fleet_inner.fleet_stats().detector_radius,
                );

                acc.intersect(&collider, |_, other| {
                    if let Some(small_id) = rest.remove(other) {
                        // We already knew about this fleet.
                        know_fleets.fleets.push((*other, small_id));
                    } else {
                        // We detected a new fleet.
                        no_know.push(*other);
                    }

                    false
                });
            };

            // rest = fleets we previously had detected and do not anymore.
            // no_know = fleets we just gained detection and 100% do not know.
            // know_fleets.fleets = fleets we already knew and are still detected.

            let mut to_forget = Vec::from_iter(rest.values().copied());

            let mut changed = Vec::with_capacity(no_know.len() + know_fleets.fleets.len());

            // Check if already known fleets have changed.
            know_fleets.fleets.drain_filter(|(fleet_id, small_id)| {
                if let Some(fleet_index) = fleets_index_map.get(fleet_id) {
                    // TODO: make sure this does not send duplicates.
                    if fleets_inner[*fleet_index].last_change() + interval > tick() {
                        changed.push((*fleet_id, *small_id));
                    }
                    false
                } else {
                    // Fleet does not exist anymore.
                    to_forget.push(*small_id);
                    true
                }
            });

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
                .filter_map(|(fleet_id, small_id)| {
                    if let Some(fleet_index) = fleets_index_map.get(fleet_id) {
                        if fleets_inner[*fleet_index].last_change() == tick() {
                            Some((*fleet_id, *small_id))
                        } else {
                            None
                        }
                    } else {
                        // Fleet does not exist anymore.
                        // It will be handled on the next full update (see above).
                        None
                    }
                })
                .collect()
        };

        let infos: Vec<FleetInfos> = changed
            .into_iter()
            .filter_map(|(fleet_id, small_id)| {
                if let Some(&fleet_index) = fleets_index_map.get(&fleet_id) {
                    Some(FleetInfos {
                        fleet_id,
                        name: fleets_name[fleet_index].clone(),
                        orbit: fleets_orbit[fleet_index].clone(),
                        fleet_composition: fleets_inner[fleet_index].fleet_composition().clone(),
                        small_id,
                    })
                } else {
                    // Fleet does not exist anymore.
                    None
                }
            })
            .collect();

        // Send detected fleets infos the client does not know.
        if !infos.is_empty() {
            connection.send_packet_reliable(
                ServerPacket::FleetsInfos(FleetsInfos {
                    tick: tick(),
                    infos,
                })
                .serialize(),
            );
        }

        // Send needed detected fleets position.
        let origin = *client_fleet_position;
        connection.send_packet_unreliable(
            ServerPacket::FleetsPosition(FleetsPosition {
                tick: tick(),
                origin,
                relative_fleets_position: know_fleets
                    .fleets
                    .iter()
                    .filter_map(|(fleet_id, small_id)| {
                        if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                            if fleets_orbit[fleet_index].is_none() {
                                Some((*small_id, CVec2::from_vec2(fleets_position[fleet_index] - origin)))
                            } else {
                                // No need to send orbiting fleet position.
                                None
                            }
                        } else {
                            // Fleet does not exist anymore.
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
