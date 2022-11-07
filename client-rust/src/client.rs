use crate::client_battlescape::ClientBattlescape;
use crate::time_manager::TimeManagerConfig;
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

    fn new(_base: &Node2D) -> Self {
        // TODO: Try to load from file.
        let client_config = ClientConfig::default();

        let bc = ClientBattlescape::new(Default::default(), &client_config);

        Client {
            metascape: (),
            bcs: vec![bc],
            t: 0.0,
            client_config,
        }
    }

    // #[method]
    // unsafe fn _unhandled_input(&mut self, event: Ref<InputEvent>) {
    //     self.metascape_manager.unhandled_input(event.assume_safe());
    // }

    #[method]
    unsafe fn _draw(&mut self, #[base] base: &Node2D) {
        // TODO: Active bc/mc
        self.bcs[0].draw(base);
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

        // TODO: Remove. Manualy ad cmds
        self.t += delta;
        if self.t >= 1.0 / 20.0 {
            self.t -= 1.0 / 20.0;

            for bc in self.bcs.iter_mut() {
                let tick = bc.replay.cmds.len() as u64;
                bc.replay.push_cmds(tick, Default::default());
            }
        }

        for bc in self.bcs.iter_mut() {
            bc.update(delta);
        }
    }

    // #[method]
    // unsafe fn get_debug_info(&mut self) -> String {
    //     self.metascape_manager.update_debug_info = true;
    //     std::mem::take(&mut self.metascape_manager.last_debug_info)
    // }

    // #[godot]
    // unsafe fn get_client_position(&mut self) -> Vector2 {
    //     if let ClientState::Connected(client_metascape) = &mut self.client_state {
    //         if let Some(fleet_state) = client_metascape.states_manager.get_client_fleet() {
    //             fleet_state
    //                 .get_interpolated_pos(&client_metascape.time_manager)
    //                 .to_godot_scaled()
    //         } else {
    //             client_metascape.states_manager.client_position.to_godot_scaled()
    //         }
    //     } else {
    //         Vector2::ZERO
    //     }
    // }
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
            Self::FatalError => &[("err", VariantType::GodotString)],
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
