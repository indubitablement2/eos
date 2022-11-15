use super::*;

const DEFAULT_FRICTION: f32 = 0.3;
const DEFAULT_RESTITUTION: f32 = 0.2;
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 0.0;
const DEFAULT_LINEAR_DAMPING: f32 = 0.02;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.02;

pub struct SimpleRigidBodyBuilder {
    pub builder: RigidBodyBuilder,
    pub copy_group_ignore: Option<RigidBodyHandle>,
}
impl SimpleRigidBodyBuilder {
    pub fn dynamic() -> Self {
        Self {
            builder: RigidBodyBuilder::dynamic()
                .linear_damping(DEFAULT_LINEAR_DAMPING)
                .angular_damping(DEFAULT_ANGULAR_DAMPING),
            copy_group_ignore: None,
        }
    }

    pub fn kinematic_position_based() -> Self {
        Self {
            builder: RigidBodyBuilder::kinematic_position_based(),
            copy_group_ignore: None,
        }
    }

    pub fn translation(mut self, translation: na::Vector2<f32>) -> Self {
        self.builder = self.builder.translation(translation);
        self
    }

    pub fn rotation(mut self, angle: f32) -> Self {
        self.builder = self.builder.rotation(angle);
        self
    }

    pub fn linvel(mut self, linvel: na::Vector2<f32>) -> Self {
        self.builder = self.builder.linvel(linvel);
        self
    }

    pub fn angvel(mut self, angvel: f32) -> Self {
        self.builder = self.builder.angvel(angvel);
        self
    }

    pub fn copy_group_ignore(mut self, from: RigidBodyHandle) -> Self {
        self.copy_group_ignore = Some(from);
        self
    }
}

pub struct SimpleColliderBuilder {
    pub builder: ColliderBuilder,
}
impl SimpleColliderBuilder {
    pub const GROUP_SHIP: Group = Group::GROUP_1;
    pub const GROUP_SHIELD: Group = Group::GROUP_2;
    pub const GROUP_DEBRIS: Group = Group::GROUP_3;
    pub const GROUP_MISSILE: Group = Group::GROUP_4;
    pub const GROUP_FIGHTER: Group = Group::GROUP_5;
    pub const GROUP_PROJECTILE: Group = Group::GROUP_6;

    pub const GROUP_ALL: Group = Self::GROUP_SHIP
        .union(Self::GROUP_SHIELD)
        .union(Self::GROUP_DEBRIS)
        .union(Self::GROUP_MISSILE)
        .union(Self::GROUP_FIGHTER)
        .union(Self::GROUP_PROJECTILE);

    pub const GROUPS_SHIP: InteractionGroups =
        InteractionGroups::new(Self::GROUP_SHIP, Self::GROUP_ALL);

    pub fn new_ship(shape: SharedShape) -> Self {
        Self {
            builder: ColliderBuilder::new(shape)
                .collision_groups(Self::GROUPS_SHIP)
                // Need ActiveHooks::FILTER_INTERSECTION_PAIR ?
                .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
                .active_events(ActiveEvents::all())
                .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
                .friction(DEFAULT_FRICTION)
                .restitution(DEFAULT_RESTITUTION),
        }
    }

    pub fn translation(mut self, translation: na::Vector2<f32>) -> Self {
        self.builder = self.builder.translation(translation);
        self
    }

    pub fn rotation(mut self, angle: f32) -> Self {
        self.builder = self.builder.rotation(angle);
        self
    }

    pub fn density(mut self, density: f32) -> Self {
        self.builder = self.builder.density(density);
        self
    }
}
