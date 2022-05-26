use crate::serverW::*;

pub fn update_fleets_in_system(s: &mut Server) {
    // Grab the pointers to what we need to make an intersection test (just a point)
    // and update the in system component.
    let (position, in_system) =
        query_ptr!(s.fleets, Fleet::position, Fleet::in_system,);

    for i in 0..s.fleets.len() {
        let (position, in_system) = unsafe { (&*position.add(i), &mut *in_system.add(i)) };

        if let Some(system_id) = in_system {
            // We are in a system. Simply check if we are still in this system.
            if let Some(index) = s.systems.get_index(*system_id) {
                let (system_position, system_radius) = query!(
                    s.systems,
                    index,
                    System::position,
                    System::radius
                );
                if !Collider::new(*system_position, *system_radius, ())
                    .intersection_test_point(*position)
                {
                    // We are not in this system anymore.
                    *in_system = None;
                }
            } else {
                // This system does not exist anymore.
                *in_system = None;
            }
        } else {
            // We are not in a system. Make an intersection test agains all the systems.
            s.systems_acceleration_structure
                .intersect_point(*position, (), |system_collider| {
                    *in_system = Some(system_collider.id);
                    true
                });
        }
    }
}
