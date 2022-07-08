use crate::*;

pub fn handle_disconnect<C>(
    s: &mut Metascape<C>,
    mut connection: C::ConnectionType,
    reason: Option<DisconnectedReason>,
) where
    C: ConnectionsManager,
{
    let connections_manager = &mut s.connections_manager;

    if let Some(reason) = reason {
        connection.send_reliable(&ServerPacket::DisconnectedReason(reason));
        let _ = connection.flush();
    }

    connections_manager.disconnect(connection);
}
