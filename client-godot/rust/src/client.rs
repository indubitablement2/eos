use crate::client_configs::ClientConfigs;
use crate::client_metascape::*;
use crate::connection_wrapper::*;
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

struct PendingConnection {
    connection: Option<ConnectionClientSideWrapper>,
    unhandled_packets: Vec<ServerPacket>,
    /// Duration without receiving a packet.
    wait_duration: f32,
}
impl PendingConnection {
    /// We will not wait in pending connection mode for more than this many seconds
    /// without receiving a packet.
    const MAX_WAIT_DURATION: f32 = 20.0;

    fn new(connection: ConnectionClientSideWrapper) -> Self {
        Self {
            connection: Some(connection),
            unhandled_packets: Default::default(),
            wait_duration: Default::default(),
        }
    }
}

#[derive(Default)]
enum ClientState {
    Connected(ClientMetascape),
    PendingConnection(PendingConnection),
    #[default]
    Unconnected,
}

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Client {
    player_inputs: PlayerInputs,
    client_configs: ClientConfigs,
    rt: tokio::runtime::Runtime,
    client_state: ClientState,
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
            client_state: Default::default(),
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

    #[godot]
    unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
        if let ClientState::Connected(client_metascape) = &mut self.client_state {
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
    }

    /// Create an offline server and client.
    /// Return true if already connected.
    #[godot]
    unsafe fn connect_local(&mut self, client_id: u32) -> bool {
        _connect_local(self, client_id)
    }

    #[godot]
    unsafe fn starting_fleet_spawn_request(&mut self, starting_fleet_id: u32, system_id: u32, planets_offset: u32) {
        if let ClientState::Connected(client_metascape) = &mut self.client_state {
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
        if let ClientState::Connected(client_metascape) = &mut self.client_state {
            if let Some(fleet_state) = client_metascape.states_manager.get_client_fleet() {
                fleet_state
                    .get_interpolated_pos(&client_metascape.time_manager)
                    .to_godot_scaled()
            } else {
                client_metascape.states_manager.client_position.to_godot_scaled()
            }
        } else {
            Vector2::ZERO
        }
    }
}

pub fn _get_debug_infos_string(client: &mut Client) -> String {
    if let ClientState::Connected(client_metascape) = &mut client.client_state {
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
{}
            ",
            client_metascape.states_manager.client_position,
            client_metascape.states_manager.get_client_fleet().is_some(),
            client_metascape.time_manager.tick,
            client_metascape.time_manager.buffer_time_remaining(),
            client_metascape.time_manager.min_over_period,
            client_metascape.time_manager.time_dilation,
            client_metascape.time_manager.orbit_time(),
            client
                .metascape_manager
                .as_ref()
                .map(|metascape_manager| metascape_manager.last_metascape_debug_info_str.as_str())
                .unwrap_or_default()
        )
    } else {
        Default::default()
    }
}

unsafe fn _update(client: &mut Client, owner: &Node2D, mut delta: f32) {
    // Somehow delta can be negative...
    delta = delta.clamp(0.0, 1.0);

    client.player_inputs.update(owner);

    match &mut client.client_state {
        ClientState::Connected(client_metascape) => {
            // Handle the signals from the client metascape.
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
        ClientState::PendingConnection(pending_connection) => {
            pending_connection.wait_duration += delta;

            // Try get the login accepted packet.
            let mut login_failed = false;
            let mut client_metascape = None;
            if let Some(connection) = &mut pending_connection.connection {
                let mut login_accepted: Option<LoginAccepted> = None;

                connection.recv_packets(|packet| match &packet {
                    ServerPacket::Invalid | ServerPacket::DisconnectedReason(_) => {
                        log::error!("{:?} while awaiting a login response.", packet);
                        login_failed = true;
                    }
                    ServerPacket::ConnectionQueueLen(_) => {}
                    ServerPacket::LoginResponse(response) => {
                        if let LoginResponse::Accepted(response) = response {
                            login_accepted = Some(response.to_owned());
                        } else {
                            log::error!("{:?} while awaiting a login response.", packet);
                            login_failed = true;
                        }
                    }
                    _ => pending_connection.unhandled_packets.push(packet),
                });

                if let Some(login_accepted) = login_accepted {
                    // Create the client metascape.

                    let connection = pending_connection.connection.take().unwrap();

                    // TODO: Mods/data manager.
                    let file = File::new();
                    file.open(crate::constants::SYSTEMS_FILE_PATH, File::READ).unwrap();
                    let buffer = file.get_buffer(file.get_len());
                    file.close();
                    let systems = bincode::deserialize::<metascape::Systems>(&buffer.read()).unwrap();

                    client_metascape = Some(ClientMetascape::new(
                        connection,
                        login_accepted,
                        ClientConfigs::default(),
                        systems,
                    ));
                }
            } else {
                login_failed = true;
            }

            if login_failed {
                client.client_state = ClientState::Unconnected;
                log::warn!("Connection failed.");
                owner.emit_signal("ConnectionResult", &[false.to_variant()]);
            } else if pending_connection.wait_duration > PendingConnection::MAX_WAIT_DURATION {
                client.client_state = ClientState::Unconnected;
                log::warn!("Connection attempt timedout.");
                owner.emit_signal("ConnectionResult", &[false.to_variant()]);
            } else if let Some(client_metascape) = client_metascape {
                client.client_state = ClientState::Connected(client_metascape);
                log::info!("Connection successful");
                owner.emit_signal("ConnectionResult", &[true.to_variant()]);
            }
        }
        ClientState::Unconnected => {}
    }

    owner.update();
}

/// Return if already connected or pending connection.
unsafe fn _connect_local(client: &mut Client, client_id: u32) -> bool {
    if let ClientState::Unconnected = &client.client_state {
        let client_id = ClientId(client_id);

        let (cc, mc) = OfflineConnectionClientSide::new();
        let offline_connections_manager = OfflineConnectionsManager {
            pending_connection: Some((Auth::Local(client_id), mc)),
        };

        client.client_state =
            ClientState::PendingConnection(PendingConnection::new(ConnectionClientSideWrapper::Offline(cc)));

        // TODO: Mods/data manager.
        let file = File::new();
        file.open(crate::constants::SYSTEMS_FILE_PATH, File::READ).unwrap();
        let buffer = file.get_buffer(file.get_len());
        file.close();
        let systems = bincode::deserialize::<metascape::Systems>(&buffer.read()).unwrap();

        let metascape = Metascape::new(Default::default(), systems.clone(), Default::default());
        let metascape_manager = MetascapeManager::new(metascape, offline_connections_manager);

        client.metascape_manager = Some(metascape_manager);
    }

    true
}
