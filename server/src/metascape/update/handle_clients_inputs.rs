use crate::metascape::*;

/// Handle the clients inbound packets.
///
/// Handle disconnected clients.
pub fn handle_clients_inputs(s: &mut Metascape) {
    let connection = query_ptr!(s.clients, Client::connection).0;

    let mut disconnected = Vec::new();

    for i in 0..s.clients.len() {
        let (connection,) = unsafe { (&*connection.add(i),) };

        let client_id = connection.client_id();

        // Get the client's fleet's index as we will likely need it.
        let fleet_index = if let Some(i) = s.fleets.get_index(connection.client_id().to_fleet_id())
        {
            i
        } else {
            log::error!("{:?} is connected, but can not find it's fleet.", client_id);
            continue;
        };
        // A place to cache the fleet's wish position.
        let mut fleet_wish_position: Option<&mut WishPosition> = None;

        // Handle the client's packets.
        loop {
            match connection.try_recv() {
                Ok(buffer) => match ClientPacket::deserialize(&buffer) {
                    ClientPacket::Invalid => {
                        debug!("{:?} sent an invalid packet. Disconnecting...", client_id);
                        disconnected.push((client_id, fleet_index));
                        break;
                    }
                    ClientPacket::MetascapeWishPos {
                        wish_pos,
                        movement_multiplier,
                    } => {
                        if let Some(fleet_wish_position) = &mut fleet_wish_position {
                            fleet_wish_position.set_wish_position(wish_pos, movement_multiplier);
                        } else {
                            fleet_wish_position =
                                Some(query!(s.fleets, fleet_index, mut Fleet::wish_position).0);
                        }
                    }
                    ClientPacket::BattlescapeInput {
                        wish_input,
                        last_acknowledge_command,
                    } => {
                        todo!()
                    }
                },
                Err(err) => match err {
                    crossbeam::channel::TryRecvError::Empty => {
                        break;
                    }
                    crossbeam::channel::TryRecvError::Disconnected => {
                        disconnected.push((client_id, fleet_index));
                    }
                },
            }
        }
    }

    // Handle disconnected clients.
    for (client_id, fleet_index) in disconnected.into_iter() {
        s.clients
            .swap_remove_by_id(client_id)
            .expect("There should be a client");

        //TODO: Set the client's fleet ai to what he chose.
        *&mut *query!(s.fleets, fleet_index, mut Fleet::fleet_ai).0 = FleetAi::Idle;
    }
}
