pub mod battlescape_config;
pub mod battlescape_inner;
mod runner;
mod signal;
mod time_manager;

use gdnative::api::*;
use gdnative::prelude::*;

use self::battlescape_config::BattlescapeConfig;
use self::battlescape_inner::*;
use self::runner::RunnerHandle;
use self::signal::BattlescapeSignal;
use self::time_manager::*;

/// The expected real world time duration of a `Battlescape` tick. 20 ups
pub const BATTLESCAPE_TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(50);
pub const BATTLESCAPE_TICK_DURATION_MIL: u32 = BATTLESCAPE_TICK_DURATION.as_millis() as u32;

#[derive(Default, Clone, Copy, Debug)]
enum ClientState {
    #[default]
    Server,
    Client,
}

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Battlescape {
    take_save: bool,

    client_state: ClientState,
    time_manager: TimeManager<BATTLESCAPE_TICK_DURATION_MIL>,
    runner_handle: RunnerHandle,
    battlescape_config: BattlescapeConfig,
}
#[methods]
impl Battlescape {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        BattlescapeSignal::register_signal(builder);
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        // TODO: Try to load from disk.
        let battlescape_config = BattlescapeConfig::default();

        Self {
            take_save: false,
            client_state: Default::default(),
            time_manager: TimeManager::new(
                battlescape_config.server_time_manager_configs.to_owned(),
            ),
            runner_handle: Default::default(),
            battlescape_config,
        }
    }

    // #[method]
    // unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
    //     self.metascape_manager.draw(owner);
    // }

    #[method]
    unsafe fn _process(&mut self, #[base] owner: &Node2D, delta: f32) {
        // Somehow delta can be negative...
        let delta = delta.clamp(0.0, 1.0);

        self.time_manager.update(delta);

        if let Some(bc) = self.runner_handle.update() {
            // Take save.
            if self.take_save {
                let save = bc.save();
                // TODO: Save to disk.
                self.take_save = false;
            }
        }
    }

    #[method]
    unsafe fn add_script(&mut self) -> i64 {
        0
    }
}
