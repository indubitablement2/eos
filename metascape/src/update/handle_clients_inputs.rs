use common::timef::TimeF;
use crate::*;

/// Handle the clients inbound packets.
///
/// Return disconnected clients.
pub fn handle_clients_inputs<C>(
    s: &mut Metascape<C>,
) -> (Vec<C::ConnectionType>, Vec<DisconnectedReason>)
where
    C: ConnectionsManager,
{
    let mut rng = &mut s.rng;
    let systems = &s.systems;
    let data = data();
    let orbit_time = TimeF::tick_to_orbit_time(tick());

    let connections = &mut s.connections;

    let fleets_wish_position = s.fleets.container.wish_position.as_mut_slice();
    let fleets_index_map = &s.fleets.index_map;

    let mut reasons = Vec::new();
    let disconnect = connections
        .drain_filter(|client_id, connection| {
            let mut reason = None;

            connection.connection.recv_packets(|packet| {
                match packet {
                    ClientPacket::ClientInputs { last_metascape_state_ack, inputs } => {
                        // Remove acked order change.
                        connection.know_fleets.non_ack_change.drain_filter(|change| change.change_tick <= last_metascape_state_ack);

                        // Handle inputs.
                        match inputs {
                            ClientInputsType::None => {}
                            ClientInputsType::Metascape { wish_pos, movement_multiplier } => {
                                if let Some(&fleet_index) = fleets_index_map.get(&client_id.to_fleet_id()) {
                                    fleets_wish_position[fleet_index]
                                        .set_wish_position(wish_pos, movement_multiplier);
                                }
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
                                FleetBuilder::new(position, fleet_composition.to_owned())
                                    .build_client(*client_id);
                            } else {
                                // Invalid starting fleet. Data might be out of sync.
                                log::debug!(
                                    "{:?} sent invalid starting fleet request. Disconnecting...",
                                    client_id
                                );
                                reason = Some(DisconnectedReason::DataOutOfSync);
                            }
                        } else {
                            log::debug!(
                                "Received fleet spawn request from {:?} who already control a fleet. Ignoring...",
                                client_id
                            );
                        }
                    }
                    ClientPacket::Invalid => {
                        log::debug!("{:?} sent an invalid packet. Disconnecting...", client_id);
                        reason = Some(DisconnectedReason::InvalidPacket);
                    }
                    ClientPacket::LoginPacket(_) => {
                        log::debug!(
                            "{:?} sent a login packet while already loged-in. Disconnecting...",
                            client_id
                        );
                        reason = Some(DisconnectedReason::InvalidPacket);
                    }
                }
            });

            if let Some(reason) = reason {
                reasons.push(reason);
                true
            } else {
                false
            }
        })
        .map(|(_, connection)| connection.connection)
        .collect();

    (disconnect, reasons)
}
