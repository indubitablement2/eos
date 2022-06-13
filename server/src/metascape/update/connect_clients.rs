use crate::metascape::*;

/// Handle new connections.
///
/// Add a default fleet/faction if client is new.
/// Otherwise client simply retake control of his fleet.
pub fn connect_clients(s: &mut Metascape) {
    let fleets_index_map = &s.fleets.index_map;

    // Fetch new connection from the `ConnectionsManager`
    // and append them at the end of the pendings queue.
    while let Ok(new_connection) = s.connections_manager.new_connection_receiver.try_recv() {
        s.pendings_connection.push_back(new_connection);
    }

    // Check if we should be updating the connection queue.
    if s.pendings_connection.len()
        > s.server_configs
            .connection_configs
            .min_pending_queue_size_for_update
        && time().tick
            % s.server_configs
                .connection_configs
                .connection_queue_update_interval
            == 0
    {
        // Check for disconnect while sending queue size.

        let mut disconnected = Vec::new();

        // Send the queue lenght to all queued connection.
        for (connection, len) in s.pendings_connection.iter().zip(0u32..) {
            if connection.send_packet_reliable(
                ServerPacket::ConnectionQueueLen(ConnectionQueueLen { len }).serialize(),
            ) {
                disconnected.push(len);
            }
        }

        // Remove disconnected connections from the queue.
        for i in disconnected.into_iter().rev() {
            s.pendings_connection.swap_remove_back(i as usize);
        }
    }

    // Handle a few connection from the pending queue.
    for _ in 0..s
        .server_configs
        .connection_configs
        .max_connection_handled_per_update
    {
        // Get a new connection.
        let connection = if let Some(connection) = s.pendings_connection.pop_front() {
            connection
        } else {
            break;
        };

        let client_id = connection.client_id();

        // Notice the client if he does not have a fleet.
        if !fleets_index_map.contains_key(&client_id.to_fleet_id()) {
            connection.send_packet_reliable(ServerPacket::NoFleet.serialize());
            log::debug!("{:?} has no fleet. Sending notification...", client_id);
        }

        // Insert client.
        let client = ClientBuilder::new(connection).build();
        if let Some(old_client) = s.clients.insert(client_id, client).1 {
            // A new connection took this client.
            log::debug!(
                "{:?} was disconnected as a new connection took this client.",
                client_id
            );

            // Send message to old connection explaining why he got disconnected.
            old_client.connection.send_packet_reliable(
                ServerPacket::DisconnectedReason(DisconnectedReasonEnum::ConnectionFromOther)
                    .serialize(),
            );
            old_client.connection.flush_tcp_stream();
        }
    }
}
