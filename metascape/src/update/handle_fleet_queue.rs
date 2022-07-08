use crate::*;

/// Insert fleets that were queued.
pub fn handle_fleet_queue<C>(s: &mut Metascape<C>)
where
    C: ConnectionsManager,
{
    let fleets = &mut s.fleets;
    let connections = &mut s.connections;

    while let Some((fleet_id, fleet)) = FLEET_QUEUE.pop() {
        // If this is for a client, also make sure the fleet info is sent to him.
        if let Some(client_id) = fleet_id.to_client_id() {
            if let Some(connection) = connections.get_mut(&client_id) {
                connection.know_fleets.update_client = true;
            }
        }

        if fleets.insert(fleet_id, fleet).1.is_some() {
            log::warn!("{:?} overwritten.", fleet_id);
        };
    }
}
