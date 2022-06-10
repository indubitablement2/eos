use crate::_metascape2::*;

/// Handle the clients inbound packets.
///
/// Remove disconnected clients.
pub fn handle_clients_inputs(s: &mut Metascape) {
    let mut disconnected = Vec::new();

    for connection in s.clients.container.connection.iter() {
        let client_id = connection.client_id();

        // Get the client's fleet's index as we will likely need it.
        let fleet_index = if let Some(i) = s.fleets.get_index(client_id.to_fleet_id()) {
            i
        } else {
            // TODO: It should be valid for a client to not have a fleet (just logged, new client, destroyed, ...).
            log::error!("{:?} is connected, but can not find it's fleet.", client_id);
            continue;
        };

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
                        s.fleets.container.wish_position[fleet_index]
                            .set_wish_position(wish_pos, movement_multiplier);
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
        s.clients
            .swap_remove_by_id(client_id)
            .expect("There should be a client");
    }
}
