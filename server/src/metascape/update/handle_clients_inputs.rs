use common::timef::TimeF;

use crate::metascape::*;

/// Handle the clients inbound packets.
///
/// Remove disconnected clients.
pub fn handle_clients_inputs(s: &mut Metascape) {
    let clients = &mut s.clients;
    let rng = &mut s.rng;
    let systems = &s.systems;
    let data = data();
    let orbit_time = TimeF::tick_to_orbit_time(tick());

    let fleets_wish_position = s.fleets.container.wish_position.as_mut_slice();
    let fleets_index_map = &s.fleets.index_map;

    let mut disconnected = Vec::new();

    for connection in clients.container.connection.iter() {
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
                        if let Some(&fleet_index) = fleets_index_map.get(&client_id.to_fleet_id()) {
                            fleets_wish_position[fleet_index]
                                .set_wish_position(wish_pos, movement_multiplier);
                        }
                    }
                    ClientPacket::CreateStartingFleet {
                        starting_fleet_id,
                        location,
                    } => {
                        if fleets_index_map.get(&client_id.to_fleet_id()).is_some() {
                            // Already have a fleet.
                            continue;
                        }

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
                            .build_client(client_id);
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

    // Remove disconnected clients.
    for client_id in disconnected.into_iter() {
        clients
            .swap_remove_by_id(client_id)
            .expect("There should be a client");
        log::debug!("Removed {:?} from metascape.", client_id)
    }
}
