use crate::client_metascape::Metascape;
use crate::configs::Configs;
use crate::connection_manager::ConnectionAttempt;
use crate::debug_infos::DebugInfos;
use crate::input_handler::PlayerInputs;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Client {
    player_inputs: PlayerInputs,
    configs: Configs,
    metascape: Option<Metascape>,
    connection_attempt: Option<ConnectionAttempt>,
    debug_infos: DebugInfos,
}

#[methods]
impl Client {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        builder
            .signal("ConnectionResult")
            .with_param("Result", VariantType::Bool)
            .done();
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        let debug_infos = DebugInfos::new();

        Client {
            player_inputs: PlayerInputs::default(),
            configs: Configs::default(),
            metascape: None,
            connection_attempt: None,
            debug_infos,
        }
    }

    #[export]
    unsafe fn _unhandled_input(&mut self, _owner: &Node2D, event: Ref<InputEvent>) {
        let event = event.assume_safe();
        self.player_inputs.handle_input(event);
    }

    #[export]
    unsafe fn _ready(&mut self, owner: &Node2D) {
        owner.add_user_signal("ConnectionResult", VariantArray::new_shared());
    }

    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {}

    #[export]
    unsafe fn _process(&mut self, owner: &Node2D, mut delta: f32) {
        // Connection attempt.
        if let Some(attempt) = self.connection_attempt.take() {
            match attempt.try_receive_result() {
                Ok(connection) => {
                    info!("Connection to server successful. Starting metascape...");
                    match Metascape::new(connection, self.configs) {
                        Ok(new_metascape) => {
                            info!("Successfully created metascape.");

                            self.metascape = Some(new_metascape);

                            owner.emit_signal("ConnectionResult", &[true.to_variant()]);
                        }
                        Err(err) => {
                            error!("{:?} while creating metascape. Aborting...", err);

                            owner.emit_signal("ConnectionResult", &[false.to_variant()]);
                        }
                    }
                }
                Err(err) => match err {
                    Ok(attempt) => {
                        self.connection_attempt = Some(attempt);
                    }
                    Err(err) => {
                        warn!("Connection attempt failed with ({:?}).", err);

                        owner.emit_signal("ConnectionResult", &[false.to_variant()]);
                    }
                },
            }
            return;
        }

        // Somehow delta can be negative...
        delta = delta.clamp(0.0, 1.0);

        self.player_inputs.update(owner);

        if let Some(metascape) = &mut self.metascape {
            self.debug_infos.update(metascape);

            if metascape.update(delta, &self.player_inputs) {
                info!("Terminated metascape as signaled.");
                self.metascape = None;
            }
        }

        // TODO: Remove rendering from draw,
        owner.update();
    }

    #[export]
    unsafe fn _draw(&mut self, owner: &Node2D) {
        if let Some(metascape) = &mut self.metascape {
            metascape.render(owner);
        }
    }

    /// Try to connect to the server.
    /// Return true if already connected.
    /// TODO: We may have to split token into two godot "int".
    #[export]
    unsafe fn connect_to_server(&mut self, _owner: &Node2D, addr: String, token: u64) -> bool {
        if self.metascape.is_some() {
            return true;
        } else {
            match ConnectionAttempt::start_login(addr.as_str(), token) {
                Ok(new_connection_attemp) => {
                    if self.connection_attempt.replace(new_connection_attemp).is_some() {
                        info!("Started a new connection attempt and dropped the previous one in progress.");
                    } else {
                        info!("Started a new connection attempt.");
                    }
                }
                Err(err) => {
                    error!("{:?} while starting a new connection attempt. Aborting...", err);
                }
            }
        }

        false
    }

    #[export]
    unsafe fn _on_draw_time_dilation(&mut self, _owner: &Node2D, control: Ref<Control>) {
        self.debug_infos.draw_time_dilation(control);
    }

    #[export]
    unsafe fn _on_draw_tick_buffer(&mut self, _owner: &Node2D, control: Ref<Control>) {
        self.debug_infos.draw_tick_buffer(control);
    }
}
