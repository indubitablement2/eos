use crate::constants::*;
use crate::ecs_resources::*;
use crate::ecs_systems::*;
use bevy_ecs::prelude::*;
use gdnative::prelude::*;
use std::mem::take;

pub struct EcsUpdateResult {
    pub render_res: RenderRes,
}

pub struct Ecs {
    pub world: World,
    schedule: Schedule,
}

impl Ecs {
    /// Init the ecs by creating a new world (or loading one) and a default schedule.
    pub fn new() -> Self {
        Self {
            world: init_world(),
            schedule: init_schedule(),
        }
    }

    /// Update the ecs.
    pub unsafe fn update(&mut self, delta: f32) -> EcsUpdateResult {
        self.pre_update(delta);
        self.schedule.run_once(&mut self.world);
        self.post_update()
    }

    /// Prepare to update.
    unsafe fn pre_update(&mut self, delta: f32) {
        // Set delta.
        self.world.get_resource_unchecked_mut::<TimeRes>().unwrap().delta = delta.into();

        // Prepare render data array.
        let mut render_res = self.world.get_resource_unchecked_mut::<RenderRes>().unwrap();
        render_res.render_data.resize(NUM_RENDER * 12);
    }

    /// Finish update and fetch render data.
    unsafe fn post_update(&mut self) -> EcsUpdateResult {
        

        EcsUpdateResult {
            // Take the render data from the ecs and replace it with default.
            render_res: take(&mut self.world.get_resource_unchecked_mut::<RenderRes>().unwrap()),
        }
    }
}

fn init_world() -> World {
    let mut world = World::default();

    // Insert other resources.
    world.insert_resource(TimeRes::default());
    world.insert_resource(GameParameterRes::default());

    // Render resource.
    world.insert_resource(RenderRes {
        render_data: TypedArray::new(),
        visible_instance: 0,
    });

    world
}

/// Create a new default schedule.
fn init_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    // Insert stages.
    schedule.add_stage("pre_update", SystemStage::parallel());
    schedule.add_system_to_stage("pre_update", time_system.system());

    schedule.add_stage_after("pre_update", "update", SystemStage::parallel());

    schedule.add_stage_after("update", "post_update", SystemStage::single_threaded());

    schedule
}
