use crate::client_metascape::Metascape;
use crate::configs::Configs;
use crate::connection_manager::ConnectionAttempt;
use crate::input_handler::PlayerInputs;
use common::idx::*;
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
}

#[methods]
impl Client {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        builder
            .signal("ConnectionResult")
            .with_param("result", VariantType::Bool)
            .done();
        builder
            .signal("Disconnected")
            .with_param("reason", VariantType::GodotString)
            .done();
        builder
            .signal("HasFleetChanged")
            .with_param("has_fleet", VariantType::Bool)
            .done();
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Client {
            player_inputs: PlayerInputs::default(),
            configs: Configs::default(),
            metascape: None,
            connection_attempt: None,
        }
    }

    #[godot]
    unsafe fn _unhandled_input(&mut self, event: Ref<InputEvent>) {
        let event = event.assume_safe();
        self.player_inputs.handle_input(event);
    }

    #[godot]
    unsafe fn _ready(&mut self, #[base] owner: &Node2D) {
        owner.add_user_signal("ConnectionResult", VariantArray::new_shared());
    }

    #[godot]
    unsafe fn _exit_tree(&mut self) {}

    #[godot]
    unsafe fn _process(&mut self, #[base] owner: &Node2D, mut delta: f32) {
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
            for metascape_signal in metascape.update(delta, &self.player_inputs) {
                match metascape_signal {
                    crate::client_metascape::MetascapeSignal::Disconnected { reason } => {
                        let reason_str = reason.to_string();
                        log::info!("Disconnected: {}", &reason_str);
                        owner.emit_signal("Disconnected", &[reason_str.to_variant()]);
                    }
                    crate::client_metascape::MetascapeSignal::HasFleetChanged { has_fleet } => {
                        log::info!("Has fleet changed: {}", has_fleet);
                        owner.emit_signal("HasFleetChanged", &[has_fleet.to_variant()]);
                    }
                }
            }
        }

        // TODO: Remove rendering from draw,
        owner.update();
    }

    #[godot]
    unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
        if let Some(metascape) = &mut self.metascape {
            metascape.render(owner);
        }
    }

    /// Try to connect to the server.
    /// Return true if already connected.
    /// TODO: We may have to split token into two godot "int".
    #[godot]
    unsafe fn connect_to_server(&mut self, addr: String, token: u64) -> bool {
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

    #[godot]
    unsafe fn starting_fleet_spawn_request(&mut self, starting_fleet_id: u32, system_id: u32, planets_offset: u32) {
        if let Some(metascape) = &self.metascape {
            let starting_fleet_id = StartingFleetId::from_raw(starting_fleet_id);
            let location = PlanetId {
                system_id: SystemId(system_id),
                planets_offset,
            };

            metascape
                .connection_manager
                .send(&common::net::packets::ClientPacket::CreateStartingFleet {
                    starting_fleet_id,
                    location,
                });

            log::debug!("Sent spawn request for {:?}.", starting_fleet_id);
        } else {
            log::warn!("Can not send starting fleet spawn request without a metascape. Aborting...");
        }
    }

    #[godot]
    unsafe fn get_debug_infos_string(&mut self) -> String {
        if let Some(metascape) = &self.metascape {
            debug_infos(metascape)
        } else {
            "No metascape".to_string()
        }
    }
}

pub fn debug_infos(metascape: &Metascape) -> String {
    format!(
        "TIME:
        Tick: {}
        Buffer time remaining: {}
        Min buffer time remaining recently: {}
        Time dilation: {}
        Orbit time: {:.2}",
        metascape.time_manager.tick,
        metascape.time_manager.buffer_time_remaining(),
        metascape.time_manager.min_over_period,
        metascape.time_manager.time_dilation,
        metascape.time_manager.orbit_time(),
    )
}
