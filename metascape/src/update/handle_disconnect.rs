use super::*;

pub fn handle_disconnect<C>(
    connections_manager: &mut C,
    disconnect: &mut Vec<(C::ConnectionType, Option<DisconnectedReason>)>,
) where
    C: ConnectionsManager,
{
    for (mut connection, reason) in disconnect.drain(..) {
        if let Some(reason) = reason {
            connection.send_reliable(&ServerPacket::DisconnectedReason(reason));
            let _ = connection.flush();
        }
        connections_manager.disconnect(connection)
    }
}
