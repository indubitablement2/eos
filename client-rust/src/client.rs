use crate::client_battlescape::ClientBattlescape;
use crate::time_manager::TimeManagerConfig;
use battlescape::commands::*;
use common::{fleet::Ship, *};
use gdnative::api::*;
use gdnative::prelude::*;

pub static FATAL_ERROR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Client {
    client_config: ClientConfig,
    metascape: (),
    bcs: Vec<ClientBattlescape>,
    t: f32,
}
#[methods]
impl Client {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        ClientSignal::register_signal(builder);
    }

    fn new(base: &Node2D) -> Self {
        // TODO: Try to load from file.
        let client_config = ClientConfig::default();

        Client {
            metascape: (),
            bcs: Default::default(),
            t: 0.0,
            client_config,
        }
    }

    // #[method]
    // unsafe fn _unhandled_input(&mut self, event: Ref<InputEvent>) {
    //     self.metascape_manager.unhandled_input(event.assume_safe());
    // }

    #[method]
    unsafe fn _ready(&mut self, #[base] base: &Node2D) {
        let cmds = (0..16)
            .map(|i| {
                BattlescapeCommand::AddFleet(AddFleet {
                    fleet_id: FleetId(i),
                    fleet: common::fleet::Fleet {
                        ships: (0..4)
                            .map(|_| Ship {
                                ship_data_id: rand::random(),
                            })
                            .collect(),
                        owner: Some(ClientId(i)),
                    },
                    team: None,
                })
            })
            .collect();
        let mut replay = Replay::default();
        replay.push_cmds(
            0,
            FullCmds {
                jump_point: None,
                cmds,
            },
        );
        let bc = ClientBattlescape::new(base, replay, &self.client_config);
        self.bcs.push(bc);
    }

    #[method]
    unsafe fn _process(&mut self, #[base] base: &Node2D, delta: f32) {
        // Handle fatal error.
        if FATAL_ERROR.load(std::sync::atomic::Ordering::Relaxed) {
            ClientSignal::FatalError.emit_signal(base);
            return;
        }

        // Somehow delta can be negative...
        let delta = delta.clamp(0.0, 1.0);

        // TODO: Remove. Manualy added cmds
        self.t += delta;
        if self.t >= 1.0 / 20.0 {
            self.t -= 1.0 / 20.0;

            for bc in self.bcs.iter_mut() {
                let input = Input::godot_singleton();
                let cmds = FullCmds {
                    jump_point: None,
                    cmds: vec![
                        BattlescapeCommand::SetClientControl(SetClientControl {
                            client_id: ClientId(0),
                            ship_id: Some(battlescape::ShipId(0)),
                        }),
                        BattlescapeCommand::SetClientInput(SetClientInput {
                            client_id: ClientId(0),
                            inputs: battlescape::bc_client::PlayerInput {
                                wish_rot: input.get_action_strength("right", false) as f32
                                    - input.get_action_strength("left", false) as f32,
                                wish_dir: na::vector![
                                    0.0,
                                    input.get_action_strength("down", false) as f32
                                        - input.get_action_strength("up", false) as f32
                                ],
                                wish_aim: 0.0,
                                fire_toggle: false,
                                wish_rot_absolute: false,
                                wish_dir_relative: true,
                                stop: input.is_action_pressed("stop", false),
                            },
                        }),
                    ],
                };

                let tick = bc.replay.cmds.len() as u64;
                bc.replay.push_cmds(tick, cmds);
            }
        }

        for bc in self.bcs.iter_mut() {
            bc.update(delta);
        }

        base.update();
    }

    #[method]
    unsafe fn _draw(&mut self, #[base] base: &Node2D) {
        // TODO: Active bc/mc
        self.bcs[0].draw(base);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClientConfig {
    pub system_draw_distance: f32,

    pub metascape_time_manager_config: TimeManagerConfig,
    pub battlescape_time_manager_config: TimeManagerConfig,
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            system_draw_distance: 256.0,
            metascape_time_manager_config: Default::default(),
            battlescape_time_manager_config: Default::default(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ClientSignal {
    FatalError,
    Poopi,
    Var(String),
}
impl ClientSignal {
    const fn name(&self) -> &'static str {
        match self {
            Self::FatalError => "FatalError",
            Self::Poopi => "Poopi",
            Self::Var(_) => "Var",
        }
    }

    const fn params(&self) -> &[(&str, VariantType)] {
        match self {
            Self::FatalError => &[],
            Self::Poopi => &[],
            Self::Var(_) => &[("param", VariantType::GodotString)],
        }
    }

    fn emit_signal(self, base: &Node2D) {
        let signal = self.name();
        match self {
            Self::FatalError => base.emit_signal(signal, &[]),
            Self::Poopi => base.emit_signal(signal, &[]),
            Self::Var(s) => base.emit_signal(signal, &[s.owned_to_variant()]),
        };
    }

    /// Create dummy signals to call `name()` and `params()` on them.
    fn _dummy() -> [Self; std::mem::variant_count::<Self>()] {
        [Self::FatalError, Self::Poopi, Self::Var(Default::default())]
    }

    /// Automaticaly register all signals.
    fn register_signal(builder: &ClassBuilder<Client>) {
        for s in Self::_dummy() {
            let mut b = builder.signal(s.name());
            for &(parameter_name, parameter_type) in s.params() {
                b = b.with_param(parameter_name, parameter_type)
            }
            b.done();
        }
    }
}
