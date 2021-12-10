use crate::client_metascape::ClientMetascape;
use crate::input_handler::InputHandler;
use std::net::IpAddr;
use std::net::SocketAddr;
use common::generation::GenerationParameters;
use common::packets::*;
use common::*;
use common::parameters::MetascapeParameters;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for logic.
/// This is either a client (multiplayer) or a server and a client (singleplayer).
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    input_handler: InputHandler,
    client_metascape: Option<ClientMetascape>,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Game {
            input_handler: InputHandler::default(),
            client_metascape: None,
        }
    }

    #[export]
    unsafe fn _unhandled_input(&mut self, _owner: &Node2D, event: Ref<InputEvent>) {
        let event = event.assume_safe();
        self.input_handler.handle_input(event);
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {}

    /// For some reason this gets called twice.
    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {}

    #[export]
    unsafe fn _process(&mut self, owner: &Node2D, mut delta: f64) {
        // Somehow delta can be negative...
        delta = delta.clamp(0.0, 1.0);

        self.input_handler.update(owner);

        if let Some(client_metascape) = &mut self.client_metascape {
            client_metascape.update(&self.input_handler);
        }

        // TODO: Remove rendering from draw,
        owner.update();
    }

    #[export]
    unsafe fn _physics_process(&mut self, _owner: &Node2D, _delta: f64) {}

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        if let Some(client_metascape) = &mut self.client_metascape {
            client_metascape.render(owner);
        }
    }

    // TODO: Connect to login server first.
    /// Return true when successfully connected.
    #[export]
    unsafe fn connect_client(&mut self, _owner: &Node2D, godot_addr: StringArray) -> bool {
        if self.client_metascape.is_some() {
            return true;
        } else {
            let godot_addr_read = godot_addr.read();
            for s in godot_addr_read.iter() {
                if let Ok(addr) = s.to_string().parse::<IpAddr>() {
                    let server_addresses = ServerAddresses {
                        tcp_address: SocketAddr::new(addr, SERVER_PORT),
                        udp_address: SocketAddr::new(addr, SERVER_PORT),
                    };

                    if let Ok(new_client_metascape) = ClientMetascape::new(
                        server_addresses,
                        MetascapeParameters::default(),
                        GenerationParameters::default(),
                    ) {
                        self.client_metascape.replace(new_client_metascape);
                        return true;
                    }
                }
            }
        }
        false
    }
}
