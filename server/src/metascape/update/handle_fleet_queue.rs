use crate::metascape::*;

/// Insert fleets that were queued.
pub fn handle_fleet_queue(s: &mut Metascape) {
    let fleets = &mut s.fleets;
    let owned_fleets = &mut s.owned_fleets;

    let clients_index_map = &s.clients.index_map;
    let clients_connection = s.clients.container.connection.as_slice();

    while let Some((fleet_id, fleet)) = FLEET_QUEUE.pop() {
        if let Some(client_id) = fleet.owner {
            // Add to owned fleets.
            let owned_fleets = owned_fleets.entry(client_id).or_default();
            owned_fleets.push(fleet_id);

            // Notify the client of the change, if he is connected.
            if let Some(&client_index) = clients_index_map.get(&client_id) {
                clients_connection[client_index].send_packet_reliable(
                    ServerPacket::OwnedFleets(owned_fleets.to_owned()).serialize(),
                );
            }
        }

        if fleets.insert(fleet_id, fleet).1.is_some() {
            log::warn!("{:?} overwritten.", fleet_id);
        };
    }
}
