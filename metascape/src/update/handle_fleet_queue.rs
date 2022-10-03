use super::*;

/// Insert fleets that were queued.
pub fn handle_fleet_queue(
    new_fleet_queue: NewFleetQueue,
    fleets: &mut Fleets,
    factions: &Factions,
) {
    for fleet_builder in new_fleet_queue {
        let fleet_id = fleet_builder.fleet_id;

        let mut masks = factions.get_faction(fleet_builder.faction).masks.clone();
        // TODO: Apply client masks modifier (only for client in neutral faction).
        if fleet_builder.faction.neutral() {
            if let Some(client_id) = fleet_id.to_client_id() {}
        }

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
            masks,
        };

        if fleets.insert(fleet_id, fleet).1.is_some() {
            log::warn!("{:?} overwritten.", fleet_id);
        };
    }
}
