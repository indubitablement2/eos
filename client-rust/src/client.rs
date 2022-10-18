use gdnative::api::*;
use gdnative::prelude::*;

use crate::client_battlescape::ClientBattlescape;
use crate::godot_client_config::GodotClientConfig;

#[derive(Debug, Eq, PartialEq)]
pub enum Signal {
    Poopi,
    Var(String),
}
impl Signal {
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
            Signal::Poopi => owner.emit_signal(signal, &[]),
            Signal::Var(s) => owner.emit_signal(signal, &[s.owned_to_variant()]),
        };
    }

    /// Create dummy signals to call `name()` and `params()` on them.
    fn _dummy() -> [Self; std::mem::variant_count::<Self>()] {
        [Self::Poopi, Self::Var(Default::default())]
    }

    /// Automaticaly register all signals.
    fn register_signal(builder: &ClassBuilder<Client>) {
        for s in Signal::_dummy() {
            let mut b = builder.signal(s.name());
            for &(parameter_name, parameter_type) in s.params() {
                b = b.with_param(parameter_name, parameter_type)
            }
            b.done();
        }
    }
}

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Client {
    metascape: (),
    bcs: Vec<ClientBattlescape>,
    config: GodotClientConfig,
}
#[methods]
impl Client {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        Signal::register_signal(builder);
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        // TODO: Load config from file.
        let config = GodotClientConfig::default();

        let bc = ClientBattlescape::new(config.battlescape_time_manager_config.clone(), Default::default());

        Client { metascape: (), bcs: vec![bc], config }
    }

    // #[method]
    // unsafe fn _unhandled_input(&mut self, event: Ref<InputEvent>) {
    //     self.metascape_manager.unhandled_input(event.assume_safe());
    // }

    #[method]
    unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
        // TODO: only draw the active bc.
        for bc in self.bcs.iter_mut() {
            bc.draw(owner);
        }
    }

    #[method]
    unsafe fn _process(&mut self, #[base] _owner: &Node2D, delta: f32) {
        // Somehow delta can be negative...
        let delta = delta.clamp(0.0, 1.0);
        
        for bc in self.bcs.iter_mut() {
            bc.process(delta);
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
