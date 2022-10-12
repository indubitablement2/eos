use super::*;

pub struct Hooks;
impl PhysicsHooks for Hooks {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        let a = context.colliders[context.collider1].user_data;
        let b = context.colliders[context.collider2].user_data;

        if get_user_data_ignore(a) == get_user_data_ignore(b) {
            None
        } else {
            Some(SolverFlags::COMPUTE_IMPULSES)
        }
    }

    fn filter_intersection_pair(&self, context: &PairFilterContext) -> bool {
        get_user_data_ignore(context.colliders[context.collider1].user_data)
            != get_user_data_ignore(context.colliders[context.collider2].user_data)
    }

    fn modify_solver_contacts(&self, _context: &mut ContactModificationContext) {}
}

// from high to low bits in chunk of 32
// - unused
// - arc angle (used for circle)
// - ignore generation
// - ignore id
pub fn set_user_data_ignore(ignore_rb: RigidBodyHandle) -> u128 {
    let (id, generation) = ignore_rb.into_raw_parts();
    ((generation as u128) << u32::BITS) | id as u128
}

pub fn get_user_data_ignore(user_data: u128) -> u64 {
    user_data as u64
}
