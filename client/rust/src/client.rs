use crate::client_metascape::*;
use crate::client_configs::ClientConfigs;
use crate::input_handler::PlayerInputs;
use crate::metasacpe_manager::*;
use crate::util::ToGodot;
use common::idx::*;
use common::net::offline_client::*;
use common::net::*;
use gdnative::api::*;
use gdnative::prelude::*;
use metascape::offline::OfflineConnectionsManager;
use metascape::Metascape;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Client {
    player_inputs: PlayerInputs,
    client_configs: ClientConfigs,
    rt: tokio::runtime::Runtime,
    client_metascape: Option<ClientMetascape>,
    /// Used when we are also the server.
    metascape_manager: Option<MetascapeManager>,
    // connection_attempt: Option<ConnectionAttempt>,
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
            client_configs: ClientConfigs::default(),
            client_metascape: None,
            metascape_manager: None,
            rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
            // connection_attempt: None,
        }
    }

    #[godot]
    unsafe fn _unhandled_input(&mut self, event: Ref<InputEvent>) {
        let event = event.assume_safe();
        self.player_inputs.handle_input(event);
    }

    // #[godot]
    // unsafe fn _ready(&mut self, #[base] owner: &Node2D) {
    // }

    #[godot]
    unsafe fn _exit_tree(&mut self) {}

    // #[godot]
    // unsafe fn _process(&mut self, #[base] owner: &Node2D, delta: f32) {
    //     // // Connection attempt.
    //     // if let Some(attempt) = self.connection_attempt.take() {
    //     //     match attempt.try_receive_result() {
    //     //         Ok(connection) => {
    //     //             info!("Connection to server successful. Starting metascape...");
    //     //             match Metascape::new(connection, self.configs) {
    //     //                 Ok(new_metascape) => {
    //     //                     info!("Successfully created metascape.");

    //     //                     self.metascape = Some(new_metascape);

    //     //                     owner.emit_signal("ConnectionResult", &[true.to_variant()]);
    //     //                 }
    //     //                 Err(err) => {
    //     //                     error!("{:?} while creating metascape. Aborting...", err);

    //     //                     owner.emit_signal("ConnectionResult", &[false.to_variant()]);
    //     //                 }
    //     //             }
    //     //         }
    //     //         Err(err) => match err {
    //     //             Ok(attempt) => {
    //     //                 self.connection_attempt = Some(attempt);
    //     //             }
    //     //             Err(err) => {
    //     //                 warn!("Connection attempt failed with ({:?}).", err);

    //     //                 owner.emit_signal("ConnectionResult", &[false.to_variant()]);
    //     //             }
    //     //         },
    //     //     }
    //     //     return;
    //     // }
    // }

    #[godot]
    unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
        if let Some(client_metascape) = &mut self.client_metascape {
            client_metascape.render(owner);
        }
    }

    #[godot]
    unsafe fn update(&mut self, #[base] owner: &Node2D, delta: f32) {
        _update(self, owner, delta);
    }

    /// Try to connect to the server.
    /// Return true if already connected.
    /// TODO: We may have to split token into two godot "int".
    #[godot]
    unsafe fn connect_to_server(&mut self, addr: String, token: u64) -> bool {
        todo!()
        // if self.metascape.is_some() {
        //     return true;
        // } else {
        //     match ConnectionAttempt::start_login(addr.as_str(), token) {
        //         Ok(new_connection_attemp) => {
        //             if self.connection_attempt.replace(new_connection_attemp).is_some() {
        //                 info!("Started a new connection attempt and dropped the previous one in progress.");
        //             } else {
        //                 info!("Started a new connection attempt.");
        //             }
        //         }
        //         Err(err) => {
        //             error!("{:?} while starting a new connection attempt. Aborting...", err);
        //         }
        //     }
        // }

        // false
    }

    /// Create an offline server and client.
    /// Return true if already connected.
    #[godot]
    unsafe fn connect_local(&mut self, client_id: u32) -> bool {
        _connect_local(self, client_id)
    }

    #[godot]
    unsafe fn starting_fleet_spawn_request(&mut self, starting_fleet_id: u32, system_id: u32, planets_offset: u32) {
        if let Some(client_metascape) = &mut self.client_metascape {
            let starting_fleet_id = StartingFleetId::from_raw(starting_fleet_id);
            let location = PlanetId {
                system_id: SystemId(system_id),
                planets_offset,
            };

            client_metascape
                .connection
                .send_reliable(&ClientPacket::CreateStartingFleet {
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
        _get_debug_infos_string(self)
    }

    #[godot]
    unsafe fn get_client_position(&mut self) -> Vector2 {
        if let Some(client_metascape) = &self.client_metascape {
            if let Some(fleet_state) = client_metascape.states_manager.get_client_fleet() {
                fleet_state.get_interpolated_pos(&client_metascape.time_manager).to_godot_scaled()
            } else {
                client_metascape.states_manager.client_position.to_godot_scaled()
            }
        } else {
            Vector2::ZERO
        }
    }
}

pub fn _get_debug_infos_string(client: &mut Client) -> String {
    let mut debug_info = if let Some(client_metascape) = &client.client_metascape {
        format!(
            "CLIENT:
            Position: {}
            Fleet: {}
TIME:
            Tick: {}
            Buffer remaining: {:.5}
            Min buffer remaining recently: {:.5}
            Time dilation: {:.4}
            Orbit time: {:.1}
            ",
            client_metascape.states_manager.client_position,
            client_metascape.states_manager.get_client_fleet().is_some(),
            client_metascape.time_manager.tick,
            client_metascape.time_manager.buffer_time_remaining(),
            client_metascape.time_manager.min_over_period,
            client_metascape.time_manager.time_dilation,
            client_metascape.time_manager.orbit_time(),
        )
    } else {
        "".to_string()
    };

    if let Some(metascape_manager) = &client.metascape_manager {
        debug_info.push_str(&metascape_manager.last_metascape_debug_info_str);
    }

    debug_info
}

unsafe fn _update(client: &mut Client, owner: &Node2D, mut delta: f32) {
        // Somehow delta can be negative...
        delta = delta.clamp(0.0, 1.0);

        client.player_inputs.update(owner);

        if let Some(client_metascape) = &mut client.client_metascape {
            for metascape_signal in client_metascape.update(delta, &client.player_inputs) {
                match metascape_signal {
                    MetascapeSignal::Disconnected { reason } => {
                        let reason_str = reason.to_string();
                        log::info!("Disconnected: {}", &reason_str);
                        owner.emit_signal("Disconnected", &[reason_str.to_variant()]);
                    }
                    MetascapeSignal::HasFleetChanged(has_fleet) => {
                        log::info!("Has fleet changed: {}", has_fleet);
                        owner.emit_signal("HasFleetChanged", &[has_fleet.to_variant()]);
                    }
                }
            }
        }

        owner.update();
}

unsafe fn _connect_local(client: &mut Client, client_id: u32) -> bool {
    if client.client_metascape.is_none() {
        let client_id = ClientId(client_id);

        let (c, mc) = OfflineConnectionClientSide::new();

        let offline_connections_manager = OfflineConnectionsManager {
            pending_connection: Some((Auth::Local(client_id), mc)),
        };

        // TODO: Mods/data manager.
        // Load systems from file.
        let file = File::new();
        file.open(crate::constants::SYSTEMS_FILE_PATH, File::READ).unwrap();
        let buffer = file.get_buffer(file.get_len());
        file.close();
        let systems = bincode::deserialize::<metascape::Systems>(&buffer.read()).unwrap();

        let metascape_manager = MetascapeManager::new(Metascape::load(
            offline_connections_manager,
            systems.clone(),
            metascape::configs::Configs::default(),
            metascape::MetascapeSave::default(),
        ));

        let client_metascape = ClientMetascape::new(
            ConnectionClientSideWrapper::Offline(c),
            client_id,
            ClientConfigs::default(),
            systems,
        );

        client.metascape_manager = Some(metascape_manager);
        client.client_metascape = Some(client_metascape);
    }

    true
}
