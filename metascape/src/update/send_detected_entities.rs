use crate::*;
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
/// Flush tcp buffer and return disconnected connection.
pub fn send_detected_entities<C>(
    s: &mut Metascape<C>,
    disconnect: &mut Vec<(C::ConnectionType, Option<DisconnectedReason>)>,
) where
    C: ConnectionsManager,
{
    let client_configs = &s.configs.client_configs;

    let fleets_out_detection_acc = &s.fleets_out_detection_acc;
    let fleets_in_detection_acc = &s.fleets_in_detection_acc;

    let connections = &mut s.connections;

    let fleets_index_map = &s.fleets.index_map;
    let fleets_position = s.fleets.container.position.as_slice();
    let fleets_inner = s.fleets.container.fleet_inner.as_slice();
    let fleets_in_system = s.fleets.container.in_system.as_slice();
    let fleets_name = s.fleets.container.name.as_slice();

    let interval = client_configs.client_detected_entity_update_interval;
    let step = tick() % interval;

    let to_disconnect = connections
        .drain_filter(|client_id, connection| {
            let client_fleet_id = client_id.to_fleet_id();

            // Update last client's stats.
            if let Some(&client_fleet_index) = fleets_index_map.get(&client_fleet_id) {
                connection.last_position = fleets_position[client_fleet_index];
                connection.last_detector_radius = fleets_inner[client_fleet_index]
                    .fleet_stats()
                    .detector_radius;
                connection.last_in_system = fleets_in_system[client_fleet_index];
                connection.ghost = false;
            } else {
                // We will use the stats from the last time the client has a fleet or some defult.
                connection.ghost = true;
            }

            let full_update = client_id.0 % interval == step;

            // Detect things around the client's fleet.
            let check_change_range = if full_update {
                let mut metascape_state_order_change = MetascapeStateOrderChange {
                    change_tick: tick(),
                    order_add: Vec::new(),
                    order_remove: Vec::new(),
                };

                // Create the collider we will be using.
                let collider =
                    Circle::new(connection.last_position, connection.last_detector_radius);

                // Get the proper acceleration structure.
                let acc = if let Some(system_id) = connection.last_in_system {
                    fleets_in_detection_acc.get(&system_id)
                } else {
                    Some(fleets_out_detection_acc)
                };

                // Detect nearby fleets.
                let mut detected = AHashSet::new();
                if let Some(acc) = acc {
                    acc.intersect(&collider, |_, other| {
                        if *other != client_fleet_id {
                            detected.insert(*other);
                        }

                        // Limit the number of new detected fleet.
                        if detected.len() >= client_configs.max_detected_fleet {
                            true
                        } else {
                            false
                        }
                    });
                } else {
                    // System acc not found.
                    // It may be empty or the system was removed.
                    // This will happen if the client is a ghost inside an empty system.
                    // Detect nothing then.
                }

                // Remove fleet we lost detection.
                connection.know_fleets.order.drain_filter(|fleet_id| {
                    if !detected.remove(fleet_id) {
                        // Fleet is no more detected.
                        metascape_state_order_change.order_remove.push(*fleet_id);
                        true
                    } else {
                        false
                    }
                });
                // Add fleet we just detected.
                for &fleet_id in detected
                    .iter()
                    .take(client_configs.max_new_detected_fleet_per_interval)
                {
                    connection.know_fleets.order.push(fleet_id);
                    metascape_state_order_change.order_add.push(fleet_id);
                }

                let num_new = metascape_state_order_change.order_add.len();
                if !metascape_state_order_change.order_add.is_empty()
                    || !metascape_state_order_change.order_remove.is_empty()
                {
                    connection
                        .know_fleets
                        .non_ack_change
                        .push(metascape_state_order_change);
                }

                // Compute the order checksum.
                let mut hasher = crc32fast::Hasher::new();
                hasher.update(bytemuck::cast_slice(&connection.know_fleets.order));
                connection.know_fleets.order_checksum = hasher.finalize();

                // New fleet will send full info anyway.
                connection.know_fleets.order.len() - num_new
            } else {
                connection.know_fleets.order.len()
            };

            // Check if known fleets have changed.
            let mut compositions_changed = Vec::new();
            connection.know_fleets.order[..check_change_range]
                .iter()
                // Add the client's fleet if we are not sending a full update.
                .chain(once(&client_fleet_id).filter(|_| !connection.know_fleets.update_client))
                .for_each(|fleet_id| {
                    if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                        // Check for fleet composition change.
                        let fleet_inner = &fleets_inner[fleet_index];
                        if fleet_inner.last_change() == tick() {
                            compositions_changed
                                .push((*fleet_id, fleet_inner.fleet_composition().to_owned()));
                        }
                    }
                });

            // Get the full info for new fleet.
            let new_fleets: Vec<FleetInfos> = connection.know_fleets.order[check_change_range..]
                .iter()
                // Add the client's fleet if we were requested to send a full update.
                .chain(once(&client_fleet_id).filter(|_| connection.know_fleets.update_client))
                .filter_map(|fleet_id| {
                    if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                        Some(FleetInfos {
                            fleet_id: *fleet_id,
                            name: fleets_name[fleet_index].to_owned(),
                            fleet_composition: fleets_inner[fleet_index]
                                .fleet_composition()
                                .to_owned(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            connection.know_fleets.update_client = false;

            // Send detected fleets infos the client does not know.
            if !new_fleets.is_empty() || !compositions_changed.is_empty() {
                connection
                    .connection
                    .send_reliable(&ServerPacket::FleetsInfos(FleetsInfos {
                        tick: tick(),
                        new_fleets,
                        compositions_changed,
                    }));
            }

            // Compute how many fleet position we can fit in a packet.
            let max_send = MAX_UNRELIABLE_PACKET_SIZE.saturating_sub(
                MetascapeState::BASE_SIZE
                    + connection
                        .know_fleets
                        .non_ack_change
                        .iter()
                        .fold(0, |acc, changes| acc + changes.size()),
            ) / 4;

            // Send the current metascape state.
            if max_send < 16 {
                // Use tcp which has a much higher cap.
                let relative_fleets_position = connection
                    .know_fleets
                    .order
                    .iter()
                    .map(|fleet_id| {
                        if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                            CVec2::from_vec2(
                                fleets_position[fleet_index] - connection.last_position,
                            )
                        } else {
                            // Fleet does not exist anymore.
                            // This will be interpreted as "ignore me".
                            CVec2 {
                                x: u16::MAX,
                                y: u16::MAX,
                            }
                        }
                    })
                    .collect();

                connection
                    .connection
                    .send_reliable(&ServerPacket::MetascapeState(MetascapeState {
                        tick: tick(),
                        ghost: connection.ghost,
                        order_checksum: connection.know_fleets.order_checksum,
                        non_ack_change: connection.know_fleets.non_ack_change.to_owned(),
                        origin: connection.last_position,
                        relative_fleets_position,
                        sent_order_start: 0,
                    }));
            } else {
                let num_part = connection.know_fleets.order.len() / max_send + 1;
                let part_size = max_send.min(connection.know_fleets.order.len());
                let part_number = tick() as usize % num_part;
                let order_start = part_size * part_number;
                let order_end = order_start + part_size;

                let relative_fleets_position = connection.know_fleets.order[order_start..order_end]
                    .iter()
                    .map(|fleet_id| {
                        if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                            CVec2::from_vec2(
                                fleets_position[fleet_index] - connection.last_position,
                            )
                        } else {
                            // Fleet does not exist anymore.
                            // This will be interpreted as "ignore me".
                            CVec2 {
                                x: u16::MAX,
                                y: u16::MAX,
                            }
                        }
                    })
                    .collect();

                connection
                    .connection
                    .send_unreliable(&ServerPacket::MetascapeState(MetascapeState {
                        tick: tick(),
                        ghost: connection.ghost,
                        order_checksum: connection.know_fleets.order_checksum,
                        non_ack_change: connection.know_fleets.non_ack_change.to_owned(),
                        origin: connection.last_position,
                        relative_fleets_position,
                        sent_order_start: order_start as u16,
                    }));
            }

            // Flush the tcp buffer.
            connection.connection.flush()
        })
        .map(|(_, connection)| connection.connection)
        .collect::<Vec<_>>();

    disconnect.extend(
        to_disconnect
            .into_iter()
            .map(|connection| (connection, None)),
    );
}
