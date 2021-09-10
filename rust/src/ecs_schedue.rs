use crate::ecs_resources::*;
use crate::ecs_systems::*;
use bevy_ecs::prelude::*;

pub struct EcsWorld {
    pub world: World,
    pub schedule: Schedule,
}

impl EcsWorld {
    /// Init the ecs by creating a new world or loading one.
    pub fn new() -> EcsWorld {
        EcsWorld {
            world: init_world(),
            schedule: init_schedule(),
        }
    }

    pub fn run(&mut self, delta: f32) {
        unsafe {
            let time_res: &mut TimeRes = &mut self.world.get_resource_unchecked_mut().unwrap();
            time_res.delta = delta;
        }
        self.schedule.run_once(&mut self.world);
    }
}

fn init_world() -> World {
    let mut world = World::default();

    // Insert resources.
    world.insert_resource(TimeRes {
        tick: 0,
        time_accumulator: 0.0,
        delta: 0.0,
    });
    world.insert_resource(GameParameterRes { drag: 0.75 });

    world
}

/// Create a new default schedule.
fn init_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    // Insert stages.
    schedule.add_stage("pre_update", SystemStage::parallel());
    schedule.add_system_to_stage("pre_update", time_system.system());
    schedule.add_stage_after("pre_update", "update", SystemStage::parallel());

    schedule
}
