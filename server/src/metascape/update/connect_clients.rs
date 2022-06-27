use crate::metascape::*;

/// Handle new connections and clients.
pub fn connect_clients(s: &mut Metascape) {
    let connections_manager = &s.connections_manager;
    let pendings_connection = &mut s.pendings_connection;
    let connections = &mut s.connections;
    let clients = &mut s.clients;
    let server_configs = &s.server_configs;

    // Fetch new connection from the `ConnectionsManager`
    // and append them at the end of the pendings queue.
    while let Ok(new_connection) = connections_manager.new_connection_receiver.try_recv() {
        pendings_connection.push_back(new_connection);
    }

    // Check if we should be updating the connection queue.
    if pendings_connection.len()
        > server_configs
            .connection_configs
            .min_pending_queue_size_for_update
        && tick()
            % server_configs
                .connection_configs
                .connection_queue_update_interval
            == 0
    {
        // Check for disconnect while sending queue size.

        let mut disconnected = Vec::new();

        // Send the queue lenght to all queued connection.
        for (connection, len) in pendings_connection.iter().zip(0u32..) {
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
    for _ in 0..server_configs
        .connection_configs
        .max_connection_handled_per_update
    {
        // Get a new connection.
        let connection = if let Some(connection) = pendings_connection.pop_front() {
            connection
        } else {
            break;
        };

        let client_id = connection.client_id();

        // TODO: Notice the client of his fleets.
        // let fleets = owned_fleets.get(&client_id).cloned().unwrap_or_default();
        // connection.send_packet_reliable(ServerPacket::OwnedFleets(fleets).serialize());
        // log::debug!("Notified {:?} of his fleets.", client_id);

        // Insert connection.
        let connection = ConnectionBuilder::new(connection).build();
        if let Some(old_connection) = connections.insert(client_id, connection).1 {
            // A new connection took this client.
            log::debug!(
                "{:?} was disconnected as a new connection took this client.",
                client_id
            );

            // Send message to old connection explaining why he got disconnected.
            old_connection.connection.send_packet_reliable(
                ServerPacket::DisconnectedReason(DisconnectedReasonEnum::ConnectionFromOther)
                    .serialize(),
            );
            old_connection.connection.flush_tcp_stream();
        };

        // Insert client if it is new.
        if clients.get_index(client_id).is_none() {
            let client = ClientBuilder::new().build();
            clients.insert(client_id, client);
            log::debug!("First connection from {:?}.", client_id);
        }
    }
}
