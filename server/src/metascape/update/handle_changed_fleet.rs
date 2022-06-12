use crate::metascape::*;

pub fn handle_changed_fleet(s: &mut Metascape) {
    let tick = time().tick;
    for fleet_inner in s.fleets.container.fleet_inner.iter_mut() {
        if fleet_inner.last_change() >= tick {
            // TODO: More robust way to detect change.
            fleet_inner.update_stats();
        }
    }
}
