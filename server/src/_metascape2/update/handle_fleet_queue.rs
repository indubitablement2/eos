use crate::_metascape2::*;

/// Insert fleets that were queued.
pub fn handle_fleet_queue(s: &mut Metascape) {
    while let Some((fleet_id, fleet_builder)) = FLEET_QUEUE.pop() {
        let fleet = Fleet {
            faction_id: fleet_builder.faction_id,
            name: fleet_builder.name,
            fleet_inner: FleetInner::new(fleet_builder.fleet_composition),
            in_system: None,
            position: fleet_builder.position,
            velocity: fleet_builder.velocity,
            wish_position: fleet_builder.wish_position,
            orbit: None,
            idle_counter: IdleCounter::default(),
            fleet_ai: fleet_builder.fleet_ai,
        };

        if s.fleets.insert(fleet_id, fleet).1.is_some() {
            log::debug!("{:?} overwritten.", fleet_id);
        };
    }
}
