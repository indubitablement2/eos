use crate::chunk_generator::*;
use crate::ecs_resources::*;
use crate::ecs_systems::*;

use bevy_ecs::prelude::*;
use crossbeam_channel::*;

/// Parameter/command that are passed to the shedule.
///
/// Shedule wait for this before updating.
pub struct Run {
    pub query_terrain_on_update: bool,
    pub force_query_terrain: bool,
    pub query_pawn: bool,
}
/// Data requested after one update. 
///
/// Shedule will send this after each update.
pub struct PostUpdate {
    pub tick: u32,
    pub terrain: Option<Vec<u8>>,
}

pub fn init_generate(width: u16, height: u16,) -> (Sender<Run>, Receiver<PostUpdate>) {
    let mut world = World::default();
    world.insert_resource(Time{
        tick: 0
    });

    let generated_terrain = generate(
        width.into(),
        height.into(),
    );

    let terrain = Terrain{
        width,
        height,
        terrain: generated_terrain
    };

    world.insert_resource(terrain);

    start_shedule(world)
}

pub fn init_load() {

}

fn start_shedule(mut world: World) -> (Sender<Run>, Receiver<PostUpdate>) {
    // Setup sender/receiver.
    let (rs,rr) = unbounded::<Run>();
    let (pus, pur) = unbounded::<PostUpdate>();

    // Setup world.
    world.insert_resource(GameParameter{
        drag: 0.75,
    });

    // Setup schedule.
    let mut schedule = Schedule::default();

    schedule.add_stage("pre_update", SystemStage::parallel());
    schedule.add_system_to_stage("pre_update", time_system.system());

    schedule.add_stage_after("pre_update", "update", SystemStage::parallel());
    // schedule.add_system_to_stage("update", useless_system.system());
    
    // Move ecs to its own thread, so it does not block main on long update.
    std::thread::spawn(move || {runner(world, schedule, rr, pus)});

    (rs, pur)
}

/// Run the schedule in a loop.
/// Send a PostUpdate every iteration that need to be gathered.
fn runner(
    mut world: World,
    mut schedule: Schedule,
    run_receiver: crossbeam_channel::Receiver<Run>,
    post_update_sender: crossbeam_channel::Sender<PostUpdate>,
) {
    loop {
        // Wait for a new run.
        if let Ok(run) = run_receiver.recv() {
            schedule.run_once(&mut world);

            // * Gather data requested by run.
            // Terrain.
            let mut terrain = Option::None;
            if run.force_query_terrain || run.query_terrain_on_update && world.is_resource_changed::<Terrain>() {
                terrain = Some(world.get_resource::<Terrain>().unwrap().terrain.clone());
            }
            
            // * Construct PostUpdate.
            let post_update = PostUpdate {
                terrain,
                tick: world.get_resource::<Time>().unwrap().tick,
            };

            // * Blocking send PostUpdate.
            if post_update_sender.send(post_update).is_err() {
                // Channel was dropped.
                break;
            }
        } else {
            // Channel was dropped.
            break;
        }

        
    }
}