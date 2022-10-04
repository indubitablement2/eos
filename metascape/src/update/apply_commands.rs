use super::*;

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and orbit.
pub fn apply_commands(s: &mut Metascape, cmds: &[TickCmd]) {
    for (caller, cmd) in cmds {
        match cmd {
            MetascapeCommand::FleetWishPosition {
                fleet_id,
                new_wish_position,
            } => {
                if let Some(client) = s.clients.get(caller) {
                    if client.owned_fleet.contains(fleet_id) {
                        if let Some(fleet_wish_position) = get_from_id_mut(
                            fleet_id,
                            &s.fleets.index_map,
                            &mut s.fleets.container.wish_position,
                        ) {
                            *fleet_wish_position = *new_wish_position;
                        } else {
                            log::warn!(
                                "{:?} not found while attemping to change its wish position. Ignoring...",
                                fleet_id
                            );
                        }
                    } else {
                        log::warn!(
                            "{:?} tried to move {:?} which he does not own. Ignoring...",
                            caller,
                            fleet_id
                        );
                    }
                } else {
                    log::warn!("{:?} does not exist. Ignoring command...", caller);
                }
            }
            MetascapeCommand::ChatMessage { message } => {}
        }
    }
}
