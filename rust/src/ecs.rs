use crate::constants::*;
use crate::ecs_components::*;
use crate::ecs_resources::*;
use crate::ecs_systems::*;
use bevy_ecs::prelude::*;
use gdnative::prelude::*;

pub struct EcsUpdateResult {
    /// This is the bulk array and the number of visible sprites.
    pub render_data: (TypedArray<f32>, i64),
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

    /// Send an update request to the ecs.
    pub fn update(&mut self, delta: f32) -> EcsUpdateResult {
        // * Pre-update:
        // Prepare render data array.
        unsafe {
            let mut render_data = TypedArray::new();
            render_data.resize(NUM_RENDER * 12);
            self.world
                .get_resource_unchecked_mut::<RenderRes>()
                .unwrap()
                .render_data
                .replace(render_data);
        }

        // * Update:

        // * Post-update:
        // Take the render data from the ecs.
        // let mut render_data = Option::None;
        // if let Some(num_render) = update_request.send_render {
        //     let mut render_res = unsafe { world.get_resource_unchecked_mut::<RenderRes>().unwrap() };
        //     let ecs_render_data = render_res.render_data.take().unwrap_or_default();

        //     if ecs_render_data.len() == num_render * 12 {
        //         render_data.replace((ecs_render_data, render_res.visible_instance));
        //     } else {
        //         godot_warn!(
        //             "Expected render data of size {}, but got {} from ecs. Sending empty array instead.",
        //             num_render * 12,
        //             ecs_render_data.len()
        //         );
        //     }

        // }

        todo!()
    }
}

fn init_world() -> World {
    let mut world = World::default();

    // Insert other resources.
    world.insert_resource(TimeRes { tick: 0 });
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

    schedule.add_stage_after("update", "post_update", SystemStage::single_threaded());

    schedule
}
