use crate::metascape::*;


pub fn handle_changed_fleet(s: &mut Metascape) {
    let tick = time().tick;
    let closure = |_, fleet_inner: &mut FleetInner| {
        if fleet_inner.last_change() >= tick {
            fleet_inner.update_stats();
        }

        false
    };
    query_closure!(s.fleets, closure, Fleet::fleet_inner);
}