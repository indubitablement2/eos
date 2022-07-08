use crate::*;

pub fn update_fleets_in_system<C>(s: &mut Metascape<C>)
where
    C: ConnectionsManager,
{
    let position = s.fleets.container.position.iter();
    let in_system = s.fleets.container.in_system.iter_mut();

    for (position, in_system) in position.zip(in_system) {
        if let Some(system_id) = in_system {
            // We are in a system. Simply check if we are still in this system.
            if !s.systems.systems.get(&system_id).is_some_and(|system| {
                Circle::new(system.position, system.radius).intersection_test_point(*position)
            }) {
                *in_system = None;
            }
        } else {
            // We are not in a system. Make an intersection test agains all the systems.
            s.systems_acceleration_structure
                .intersect_point(*position, |_, system_id| {
                    *in_system = Some(*system_id);
                    true
                });
        }
    }
}
