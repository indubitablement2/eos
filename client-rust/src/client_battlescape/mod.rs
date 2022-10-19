mod runner;
pub mod snapshop;

use self::runner::RunnerHandle;
use self::snapshop::BattlescapeSnapshot;
use crate::shared::SHARED;
use crate::time_manager::*;
use battlescape::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct ClientBattlescape {
    /// Flag telling if we are very far behind.
    ///
    /// This will disable rendering and inputs to speed up simulation.
    catching_up: bool,
    time_manager: TimeManager<{ Battlescape::TICK_DURATION_MS }>,
    runner_handle: RunnerHandle,
    snapshot: (BattlescapeSnapshot, BattlescapeSnapshot),
    data_id: u32,
}
#[methods]
impl ClientBattlescape {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        ClientBattlescapeSignal::register_signal(builder);
    }

    fn new(base: &Node2D) -> Self {
        // Take the first available battlescape data.
        let mut w = SHARED.write();
        for (&data_id, data) in w.client_battlescape_data.iter_mut() {
            if data.taken.is_none() {
                data.taken = Some(unsafe { base.assume_shared() });

                // TODO: Take latest jump point.
                let bc = Battlescape::new(data.replay.initial_state);

                return Self {
                    catching_up: true,
                    time_manager: TimeManager::new(w.client_config.battlescape_time_manager_config),
                    runner_handle: RunnerHandle::new(bc),
                    snapshot: (Default::default(), Default::default()),
                    data_id,
                };
            }
        }

        panic!("all battlescape data taken");
    }

    #[method]
    pub unsafe fn _process(&mut self, #[base] owner: &Node2D, delta: f32) {
        let w = SHARED.read();
        if let Some(data) = w.client_battlescape_data.get(&self.data_id) {
            let mut can_advance = None;
            if let Some(bc) = self.runner_handle.update() {
                can_advance = Some(bc.tick);

                let last_tick = bc.tick.saturating_sub(1);

                self.time_manager.maybe_max_tick(last_tick);

                self.catching_up = (data.replay.cmds.len() as u64) - last_tick > 40;

                // Take snapshot for rendering.
                if !self.catching_up {
                    std::mem::swap(&mut self.snapshot.0, &mut self.snapshot.1);
                    self.snapshot.1.take_snapshot(bc);
                }

                // log::debug!(
                //     "last: {}, bc: {}, target: {}, max: {}, cmds: {}, t: {:.4}",
                //     last_tick,
                //     bc.tick,
                //     self.time_manager.tick,
                //     self.time_manager.max_tick,
                //     data.replay.cmds.len(),
                //     self.time_manager.tick as f32 + self.time_manager.tick_frac
                // );
            }

            self.time_manager.update(delta);
            log::debug!("t: {:.4}", self.time_manager.time_dilation);

            if let Some(next_tick) = can_advance {
                if let Some(cmds) = data.replay.cmds.get(next_tick as usize) {
                    // Apply jump point.
                    if let Some((bytes, _)) = &cmds.jump_point {
                        match Battlescape::load(bytes) {
                            Ok(new_bc) => {
                                self.runner_handle.bc = Some(Box::new(new_bc));
                                log::debug!("Applied jump point.");
                            }
                            Err(err) => {
                                log::error!("{:?} while loading battlescape.", err);
                            }
                        }
                    }

                    self.runner_handle.step(cmds.cmds.to_owned());
                }
            }
        } else {
            // TODO: No data?
            owner.queue_free();
        }
        owner.update();
    }

    #[method]
    pub fn _draw(&mut self, #[base] owner: &Node2D) {
        if self.catching_up {
            // TODO: Display catching up message.
        } else {
            BattlescapeSnapshot::draw_lerp(
                &self.snapshot.0,
                &self.snapshot.1,
                owner,
                self.time_manager.interpolation_weight(),
            );
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ClientBattlescapeSignal {
    Poopi,
    Var(String),
}
impl ClientBattlescapeSignal {
    const fn name(&self) -> &'static str {
        match self {
            Self::Poopi => "Poopi",
            Self::Var(_) => "Var",
        }
    }

    const fn params(&self) -> &[(&str, VariantType)] {
        match self {
            Self::Poopi => &[],
            Self::Var(_) => &[("param", VariantType::GodotString)],
        }
    }

    fn emit_signal(self, owner: &Node2D) {
        let signal = self.name();
        match self {
            Self::Poopi => owner.emit_signal(signal, &[]),
            Self::Var(s) => owner.emit_signal(signal, &[s.owned_to_variant()]),
        };
    }

    /// Create dummy signals to call `name()` and `params()` on them.
    fn _dummy() -> [Self; std::mem::variant_count::<Self>()] {
        [Self::Poopi, Self::Var(Default::default())]
    }

    /// Automaticaly register all signals.
    fn register_signal(builder: &ClassBuilder<ClientBattlescape>) {
        for s in Self::_dummy() {
            let mut b = builder.signal(s.name());
            for &(parameter_name, parameter_type) in s.params() {
                b = b.with_param(parameter_name, parameter_type)
            }
            b.done();
        }
    }
}
