use std::collections::VecDeque;
use std::convert::TryInto;
use gdnative::api::*;
use gdnative::prelude::*;
use crate::ecs_input::EcsInput;
use crate::ecs_schedue::*;

/// Layer between godot and ecs.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Sprite)]
#[register_with(Self::register_builder)]
pub struct GodotEcs {
    is_ready: bool,
    pub run_sender: flume::Sender<Run>,
    pub post_update_receiver: flume::Receiver<PostUpdate>,
    update_pending: i32,

    pending_ecs_input: VecDeque<EcsInput>,
}

#[methods]
impl GodotEcs {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {
    }

    /// The "constructor" of the class.
    fn new(_owner: &Sprite) -> Self {
        GodotEcs {
            run_sender: flume::unbounded::<Run>().0,
            is_ready: false,
            update_pending: 0,
            post_update_receiver: flume::unbounded::<PostUpdate>().1,
            pending_ecs_input: VecDeque::with_capacity(128),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Sprite) {
        // owner.set_texture(self.chunk_info.texture);
    }

    /// Render interpolated between previous and current ecs update.
    #[export]
    unsafe fn _process(&mut self, owner: &Sprite, _delta: f32) {
        // just joined:
        // local_time: u64 = current_state.time

        // on process:
        // local_time += process_delta * 1000 000 000

        // if we are the server:
        // delta_last_input += process_delta
        // while delta_last_input > 0:
        // take_input(), send_input_to_player(), delta_last_input -= fixed_update_delta
        
        // if fetch ecs_input:
        // queue in channel for async update.

        // if update_pending * fixed_update_delta > 0.25 && we are the server:
        // increase fixed_update_delta
        // take note in log, send new fixed_update_delta to player
        
        // if update_pending * fixed_update_delta > 0.5 && we are client:
        // display warning
        // if update_pending * fixed_update_delta > 2.0 && we are client:
        // fast mode (catching up message)
        // if fast_mode_time > max_fast_mode_time && we are client:
        // kick

        // while update_pending:
        // self.post_update_receiver.recv_deadline(process_start_time + (process_time * 0.5))
        // wait at most process_time * 0.5 for latest post_update?

        // if got new update:
        // playout_delay_buffer.push(update)

        // if interpolate_delta

        // current_state = fetch_ecs_state()
        // (current_state.time = previous_state.time + fixed_update_delta)

        // if local_time > current_state.time:
        // local_time = current_state.time, warn ahead
        // elif local_time < previous_state.time:
        // local_time = previous_state.time, warn behind

        // interpolate_delta = current_state.time - local_time
        // if current_state_lag < 0.05:
        // don't interpolate
        // else:
        // interpolate

        // interpolate
        // current_render_state = (current_state * vel * current_state_lag)
        // last_render_state = ...



        while let Some(_ecs_input) = self.pending_ecs_input.pop_front() {
            
        }

        // if godot time 

        // if game updated: gather render data into previous. swap previous with current.
        // prepare render.

        // Try to gather PostUpdate without blocking.
        if self.update_pending < 1 {
            // No PostUpdate to fetch.
            return;
        }

        match self.post_update_receiver.try_recv() {
            Ok(post_update) => {
                // We got a fresh new PostUpdate.
                self.update_pending -= 1;
            }
            Err(err) => {
                if err == flume::TryRecvError::Disconnected {
                    // TODO: Try to gracefully handle that.
                    godot_error!("Can not receive PostUpdate. Shedule probably panicked.");
                }
            }
        }
    }

    #[export]
    unsafe fn _physics_process(&mut self, owner: &Sprite, delta: f32) {

    }

    #[export]
    unsafe fn send_input(&mut self, owner: &Sprite) {

    }

    /// Simulate 1 tick in parallel.
    #[export]
    unsafe fn run(&mut self, _owner: &Sprite) {
        if !self.is_ready {
            godot_error!("Can not update. Chunk is not ready.");
            return;
        }

        let run = Run{
            query_terrain_on_update: true,
            force_query_terrain: false,
            query_pawn: true,
        };

        self.update_pending += 1;
        if self.run_sender.try_send(run).is_err() {
            godot_error!("Can not update. Shedule probably panicked.")
        }
    }
}
