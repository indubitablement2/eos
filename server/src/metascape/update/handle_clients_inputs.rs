use common::timef::TimeF;

use crate::metascape::*;

/// Handle the clients inbound packets.
///
/// Return disconnected clients.
pub fn handle_clients_inputs(s: &mut Metascape) -> Vec<ClientId> {
    let rng = &mut s.rng;
    let systems = &s.systems;
    let data = data();
    let orbit_time = TimeF::tick_to_orbit_time(tick());

    let connections_connection = s.connections.container.connection.as_slice();
    let clients_control = s.clients.container.control.as_mut_slice();

    let fleets_owner = s.fleets.container.owner.as_slice();
    let fleets_wish_position = s.fleets.container.wish_position.as_mut_slice();
    let fleets_index_map = &s.fleets.index_map;

    let owned_fleets = &s.owned_fleets;

    let mut disconnected = Vec::new();

    for (connection, control) in clients_connection.iter().zip(clients_control) {
        let client_id = connection.client_id();

        // Handle the client's packets.
        loop {
            match connection.try_recv() {
                Ok(buffer) => match ClientPacket::deserialize(&buffer) {
                    ClientPacket::Invalid => {
                        log::debug!("{:?} sent an invalid packet. Disconnecting...", client_id);
                        disconnected.push(client_id);
                        break;
                    }
                    ClientPacket::MetascapeWishPos {
                        wish_pos,
                        movement_multiplier,
                    } => {
                        for fleet_id in control.iter() {
                            if let Some(&fleet_index) = fleets_index_map.get(fleet_id) {
                                fleets_wish_position[fleet_index]
                                    .set_wish_position(wish_pos, movement_multiplier);
                            }
                        }
                    }
                    ClientPacket::CreateStartingFleet {
                        starting_fleet_id,
                        location,
                    } => {
                        // Get the requested data.
                        let (fleet_composition, (system, planet)) = if let Some(result) = data
                            .starting_fleets
                            .get(starting_fleet_id.id() as usize)
                            .and_then(|composition| {
                                if let Some(result) = systems.get_system_and_planet(location) {
                                    Some((composition, result))
                                } else {
                                    None
                                }
                            }) {
                            result
                        } else {
                            // Invalid starting fleet or planet id. Data might be out of sync.
                            log::debug!("{:?} sent invalid data while requesting a starting fleet. Disconnecting...", client_id);
                            disconnected.push(client_id);
                            break;
                        };

                        // We will spawn the fleet near the requested planet.
                        let position = planet
                            .relative_orbit
                            .to_position(orbit_time, system.position)
                            + rng.gen::<Vec2>() * 6.0
                            - 3.0;

                        // Create fleet.
                        FleetBuilder::new(position, fleet_composition.to_owned())
                            .with_owner(client_id)
                            .build();
                    }
                    ClientPacket::ControlOwnedFleet { fleet_id } => {
                        if let Some(fleet_id) = fleet_id {
                            // Check that the client owns the fleet.
                            if owned_fleets
                                .get(&client_id)
                                .is_some_and(|owned_fleets| owned_fleets.contains(&fleet_id))
                            {
                                *control = Some(fleet_id);
                            } else {
                                *control = None;
                            }
                        } else {
                            *control = None;
                        }

                        // Notify the client of the change.
                        connection
                            .send_packet_reliable(ServerPacket::FleetControl(*control).serialize());
                        log::debug!(
                            "{:?} control changed to {:?}. Notified client.",
                            client_id,
                            control
                        );
                    }
                },
                Err(err) => match err {
                    crossbeam::channel::TryRecvError::Empty => {
                        break;
                    }
                    crossbeam::channel::TryRecvError::Disconnected => {
                        disconnected.push(client_id);
                        break;
                    }
                },
            }
        }
    }

    disconnected
}
