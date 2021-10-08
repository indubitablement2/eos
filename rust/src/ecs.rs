use crate::ecs_resources::*;
use crate::ecs_systems::*;
use bevy_ecs::prelude::*;
use gdnative::prelude::*;

pub struct Ecs {
    pub world: World,
    schedule: Schedule,
}

impl Ecs {
    /// Init the ecs by creating a new world (or loading one) and a default schedule.
    pub fn new(canvas_rid: Rid, texture_rid: Rid) -> Self {
        Self {
            world: init_world(canvas_rid, texture_rid),
            schedule: init_schedule(),
        }
    }

    /// Update the ecs.
    /// # Safety
    /// Make sure all ecs resources are initialized and unborrowed or this may panic.
    pub unsafe fn update(&mut self, delta: f32) {
        self.pre_update(delta);
        self.schedule.run_once(&mut self.world);
        self.post_update();
    }

    /// Prepare to update.
    unsafe fn pre_update(&mut self, delta: f32) {
        // Set delta.
        self.world.get_resource_unchecked_mut::<TimeRes>().unwrap().delta = delta;
    }

    /// Finish update.
    unsafe fn post_update(&mut self) {}
}

fn init_world(canvas_rid: Rid, texture_rid: Rid) -> World {
    let mut world = World::default();

    // Insert other resources.
    world.insert_resource(TimeRes::default());
    world.insert_resource(GameParameterRes::default());
    // Render resource.
    world.insert_resource(RenderRes::new(canvas_rid, texture_rid));

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
