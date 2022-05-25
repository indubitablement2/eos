use crate::serverW::*;

pub fn update_fleets_detection_acceleration_structure(s: &mut Server) {
    // Remove the old colliders.
    s.fleets_detection_acceleration_structure.clear();

    // Grab the pointers to what we need to make a collider.
    let (position, detected_radius, in_system) = query_ptr!(
        s.fleets.container(),
        Fleet::position,
        Fleet::detected_radius,
        Fleet::in_system
    );

    // Add the new colliders.
    let iter = s
        .fleets
        .id_vec()
        .iter()
        .enumerate()
        .map(|(i, fleet_id)| unsafe {
            (
                Collider::new(
                    *position.add(i),
                    *detected_radius.add(i),
                    (*in_system.add(i))
                        .map(|system_id| system_id.0)
                        .unwrap_or(u32::MAX),
                ),
                *fleet_id,
            )
        });

    s.fleets_detection_acceleration_structure.extend(iter);

    // Update to use the new colliders.
    s.fleets_detection_acceleration_structure.update();
}
