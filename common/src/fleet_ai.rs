use bevy_ecs::prelude::*;
use glam::Vec2;

pub struct FleetAI {
    pub goal: FleetGoal,
}
impl Default for FleetAI {
    fn default() -> Self {
        Self {
            goal: FleetGoal::Idle { duration: 360 },
        }
    }
}

pub enum FleetGoal {
    /// Directly controlled by a client.
    Controlled,
    Trade {
        from: (),
        to: (),
    },
    Guard {
        who: Entity,
        radius: f32,
        duration: i32,
    },
    Idle {
        duration: i32,
    },
    Wandering {
        to: (),
        pause: i32,
    },
}
