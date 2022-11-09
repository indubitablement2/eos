use super::*;
use data::hull_data::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd, Ord, Default,
)]
pub struct HullId(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Hull {
    pub hull_data_id: HullDataId,
    pub current_defence: Defence,
    pub collider: ColliderHandle,
}

pub fn build_hull_collider(
    hull_data_id: HullDataId,
    groups: InteractionGroups,
    user_data: u128,
) -> Collider {
    let hull_data = hull_data_id.data();

    // TODO: init position
    ColliderBuilder::new(hull_data.shape.to_shared_shape())
        .density(hull_data.density)
        .friction(DEFAULT_BODY_FRICTION)
        .restitution(DEFAULT_BODY_RESTITUTION)
        .collision_groups(groups)
        .active_events(ActiveEvents::all())
        .contact_force_event_threshold(DEFAULT_FORCE_EVENT_THRESHOLD)
        .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::FILTER_INTERSECTION_PAIR)
        .user_data(user_data)
        .build()
}
