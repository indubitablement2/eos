use crate::metascape::*;

/// Insert fleets that were queued.
pub fn handle_fleet_queue(s: &mut Metascape) {
    let fleets = &mut s.fleets;

    while let Some((fleet_id, fleet)) = FLEET_QUEUE.pop() {
        if fleets.insert(fleet_id, fleet).1.is_some() {
            log::debug!("{:?} overwritten.", fleet_id);
        };
    }
}
