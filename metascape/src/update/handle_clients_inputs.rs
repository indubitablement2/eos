use super::*;
use common::timef::TimeF;

/// Handle the clients inbound packets.
///
/// Return disconnected clients.
pub fn handle_clients_inputs<C>(
    s: &mut Metascape<C>,
    disconnect: &mut Vec<(C::ConnectionType, Option<DisconnectedReason>)>,
    new_fleet_queue: &mut NewFleetQueue,
) where
    C: ConnectionsManager,
{
    let mut rng = &mut s.rng;
    let systems = &s.systems;
    let data = data();
    let total_tick = s.total_tick;
    let orbit_time = TimeF::tick_to_orbit_time(total_tick);

    let connections = &mut s.connections;

    let fleets_wish_position = s.fleets.container.wish_position.as_mut_slice();
    let fleets_index_map = &s.fleets.index_map;

    let mut to_disconnect: Vec<(ClientId, Option<DisconnectedReason>)> = Vec::new();

    for (client_id, connection) in connections.iter_mut() {
        connection.connection.recv_packets(|packet| {
            match packet {
                ClientPacket::ClientInputs { last_metascape_state_ack, inputs } => {
                    // Remove acked order change.
                    connection.know_fleets.non_ack_change.drain_filter(|change| change.change_tick <= last_metascape_state_ack);

                    // Handle inputs.
                    match inputs {
                        ClientInputsType::None => false,
                        ClientInputsType::Metascape { wish_pos, movement_multiplier } => {
                            // Make sure there is no NaN/infinity.
                            if !wish_pos.is_finite() {
                                if let Some(&fleet_index) = fleets_index_map.get(&client_id.to_fleet_id()) {
                                    fleets_wish_position[fleet_index]
                                        .set_wish_position(wish_pos, movement_multiplier);
                                }
                            } else {
                                log::debug!("Receive {} as wish pos from {:?}. Ignoring...", wish_pos, client_id);
                            }
                            false
                        }
                    }
                }
                ClientPacket::CreateStartingFleet {
                    starting_fleet_id,
                    location,
                } => {
                    // Check that the client does not already have a fleet.
                    if fleets_index_map.get(&client_id.to_fleet_id()).is_none() {
                        // Get the requested starting fleet.
                        if let Some(fleet_composition) =
                            data.starting_fleets.get(starting_fleet_id.id() as usize)
                        {
                            // TODO: Spawn fleet near a friendly planet?
                            let position = systems
                                .systems
                                .values()
                                .next()
                                .and_then(|system| {
                                    system
                                        .planets
                                        .choose(&mut rng)
                                        .map(|planet| (system, planet))
                                })
                                .map(|(system, planet)| {
                                    planet
                                        .relative_orbit
                                        .to_position(orbit_time, system.position)
                                })
                                .unwrap_or_default()
                                + rng.gen::<Vec2>() * 16.0
                                - 8.0;

                            // Create fleet.
                            new_fleet_queue.push(
                                FleetBuilder::new(client_id.to_fleet_id(), position, fleet_composition.to_owned())
                            );
                            false
                        } else {
                            // Invalid starting fleet. Data might be out of sync.
                            log::debug!(
                                "{:?} sent invalid starting fleet request. Disconnecting...",
                                client_id
                            );
                            to_disconnect.push((*client_id, Some(DisconnectedReason::DataOutOfSync)));
                            true
                        }
                    } else {
                        log::debug!(
                            "Received fleet spawn request from {:?} who already control a fleet. Ignoring...",
                            client_id
                        );
                        false
                    }
                }
                ClientPacket::Invalid => {
                    log::debug!("{:?} sent an invalid packet. Disconnecting...", client_id);
                    to_disconnect.push((*client_id, Some(DisconnectedReason::InvalidPacket)));
                    true
                }
                ClientPacket::LoginPacket(_) => {
                    log::debug!(
                        "{:?} sent a login packet while already loged-in. Disconnecting...",
                        client_id
                    );
                    to_disconnect.push((*client_id, Some(DisconnectedReason::InvalidPacket)));
                    true
                }
            }
        });
    }

    for (client_id, reason) in to_disconnect {
        if let Some(connection) = connections.remove(&client_id) {
            disconnect.push((connection.connection, reason));
        }
    }
}
