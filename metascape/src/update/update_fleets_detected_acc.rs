use crate::*;

pub fn update_fleets_detected_acc(s: &mut Metascape) {
    let fleets_position = s.fleets.container.position.as_slice();
    let fleets_fleet_inner = s.fleets.container.fleet_inner.as_slice();
    let fleets_in_system = s.fleets.container.in_system.as_slice();
    let fleets_fleet_id = s.fleets.id_vec.as_slice();

    let fleets_out_detection_acc = &mut s.fleets_out_detection_acc;
    let fleets_in_detection_acc = &mut s.fleets_in_detection_acc;

    // Add the colliders to their acc.
    for (((position, fleet_inner), in_system), fleet_id) in fleets_position
        .iter()
        .zip(fleets_fleet_inner)
        .zip(fleets_in_system)
        .zip(fleets_fleet_id)
    {
        let collider = CircleBoundingShape {
            x: position.x,
            y: position.y,
            r: fleet_inner.fleet_stats().detected_radius,
        };
        if let Some(system_id) = in_system {
            fleets_in_detection_acc
                .entry(*system_id)
                .or_default()
                .queue(*fleet_id, collider)
        } else {
            fleets_out_detection_acc.queue(*fleet_id, collider);
        }
    }

    // Update the accs.
    fleets_out_detection_acc.update();
    fleets_in_detection_acc.drain_filter(|_, acc| {
        if acc.is_future_data_empty() {
            true
        } else {
            acc.update();
            false
        }
    });
}
