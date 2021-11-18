use crate::ecs_components::*;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use glam::Vec2;


pub struct FleetAI {
    pub goal: FleetGoal,
}

pub enum FleetGoal {
    Trade {
        from: (),
        to: (),
    },
    Guard {
        who: Entity,
        radius: f32,
        duration: i32,
    },
    Wandering {
        to: (),
        pause: i32
    },
}