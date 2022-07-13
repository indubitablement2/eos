use crate::*;

/// Handle new connections and clients.
pub fn connect_clients<C>(
    s: &mut Metascape<C>,
    connections_manager: &mut C,
    disconnect: &mut Vec<(C::ConnectionType, Option<DisconnectedReason>)>,
) where
    C: ConnectionsManager,
{
    let connections = &mut s.connections;
    let clients = &mut s.clients;
    let connection_configs = &s.configs.connection_configs;
    let authenticated = &mut s.authenticated;

    // Handle a few connection from the pending queue.
    for _ in 0..connection_configs.max_connection_handled_per_update {
        let mut early_exit = true;
        let mut new_client_id = None;
        let mut new_auth = None;
        let connection = if let Some(connection) = connections_manager.get_new_login(|&auth| {
            // Add/get auth.
            let client_id = authenticated
                .entry(auth)
                .or_insert_with(|| {
                    match auth {
                        Auth::Local(client_id) => client_id,
                        Auth::SteamId(_) => {
                            log::error!("steam auth not implemented yet.");

                            // TODO: Verify with steam

                            // TODO: get a new client id
                            ClientId(0)
                        }
                    }
                })
                .to_owned();

            // Add client if it is new.
            if clients.get_index(client_id).is_none() {
                let client = ClientBuilder::new().build();
                clients.insert(client_id, client);
                log::debug!("First connection from {:?}.", client_id);
            }

            new_client_id = Some(client_id);
            new_auth = Some(auth);
            early_exit = false;
            LoginResponse::Accepted { client_id }
        }) {
            connection
        } else if early_exit {
            break;
        } else {
            continue;
        };

        let client_id = new_client_id.unwrap();
        let auth = new_auth.unwrap();

        // Insert connection.
        if let Some(old_connection) =
            connections.insert(client_id, ClientConnection::new(connection, auth))
        {
            // A new connection took this client.
            log::debug!(
                "{:?} was disconnected as a new connection took this client.",
                client_id
            );

            disconnect.push((
                old_connection.connection,
                Some(DisconnectedReason::ConnectionFromOther),
            ));
        };
    }
}
