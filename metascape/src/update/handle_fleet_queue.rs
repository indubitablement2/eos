use super::*;

/// Insert fleets that were queued.
pub fn handle_fleet_queue(
    new_fleet_queue: NewFleetQueue,
    fleets: &mut Fleets,
    clients: &mut Clients,
    factions: &mut Factions,
) {
    for mut fleet_builder in new_fleet_queue {
        let fleet_id = fleet_builder.fleet_id;

        // Add fleet to client's owned fleets.
        if let Some(client_owner) = fleet_builder.client_owner {
            if let Some(client) = clients.get_mut(&client_owner) {
                client.owned_fleet.push(fleet_id);

                // Also make sure fleet has same faction as the client.
                fleet_builder.faction = client.faction;
            } else {
                // Client does not exist.
                fleet_builder.client_owner = None;
                log::warn!("Tried to spawn a fleet owned by a client that does not exist. Removing owner...");
            }
        }

        // Add fleet to faction.
        factions
            .entry(fleet_builder.faction)
            .or_default()
            .fleets
            .insert(fleet_id);
        // TODO: Apply client masks modifier (only for client in neutral faction).
        if fleet_builder.faction.is_neutral() {}

        let name = if let Some(name) = fleet_builder.name {
            name
        } else {
            // TODO: Random name generation.
            format!("todo name generation {:?}", fleet_id)
        };

        let fleet = Fleet {
            name,
            fleet_inner: FleetInner::new(fleet_builder.fleet_composition),
            in_system: None,
            position: fleet_builder.position,
            velocity: fleet_builder.velocity,
            wish_position: Default::default(),
            orbit: None,
            idle_counter: Default::default(),
            fleet_ai: fleet_builder.fleet_ai.unwrap_or_default(),
            faction: fleet_builder.faction,
            client_owner: fleet_builder.client_owner,
        };

        if fleets.insert(fleet_id, fleet).1.is_some() {
            log::warn!("{:?} overwritten.", fleet_id);
        };
    }
}
