use crate::metascape::*;

/// Handle new connections.
///
/// Add a default fleet/faction if client is new.
/// Otherwise client simply retake control of his fleet.
pub fn connect_clients(s: &mut Metascape) {
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
        && s.time.tick
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

        // Insert client.
        let client = ClientBuilder::new(connection).build();
        if let Some(old_client) = s.clients.insert(client_id, client).1 {
            // A new connection took this client.
            debug!(
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

        let fleet_id = client_id.to_fleet_id();

        // Check if fleet is already spawned.
        if let Some(index) = s.fleets.get_index(fleet_id) {
            // Change the fleet ai.
            *query!(s.fleets, index, mut Fleet::fleet_ai).0 = FleetAi::ClientControl;
        } else {
            // Create a new faction.
            let faction_id = FactionBuilder::new().with_clients(&[client_id]).build();

            // Create a new fleet.
            FleetBuilder::new(
                faction_id,
                "insert name".to_string(),
                Vec2::ZERO,
                FleetAi::ClientControl,
                vec![],
            )
            .build_client(client_id);
        }
    }
}
