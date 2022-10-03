use crate::*;

pub fn update_fleets_in_system(s: &mut Metascape) {
    let position = s.fleets.container.position.iter();
    let in_system = s.fleets.container.in_system.iter_mut();

    for (position, in_system) in position.zip(in_system) {
        if let Some(system_id) = in_system {
            // We are in a system. Simply check if we are still in this system.
            if !s.systems.systems.get(&system_id).is_some_and(|system| {
                CircleBoundingShape {
                    x: system.position.x,
                    y: system.position.y,
                    r: system.radius,
                }
                .intersect_point(position.x, position.y)
            }) {
                *in_system = None;
            }
        } else {
            // We are not in a system. Make an intersection test agains all the systems.
            s.systems_acceleration_structure.intersect_point(
                position.x,
                position.y,
                |_, system_id| {
                    *in_system = Some(*system_id);
                    true
                },
            );
        }
    }
}
