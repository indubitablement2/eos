use crate::metasacpe_manager::*;
use gdnative::api::*;
use gdnative::prelude::*;

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
    metascape_manager: MetascapeManager,
}
#[methods]
impl Client {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(builder: &ClassBuilder<Self>) {
        Signal::register_signal(builder);
    }

    /// The "constructor" of the class.
    fn new(_owner: &Node2D) -> Self {
        Client {
            metascape_manager: MetascapeManager::new(Default::default()),
        }
    }

    #[method]
    unsafe fn _unhandled_input(&mut self, event: Ref<InputEvent>) {
        self.metascape_manager.unhandled_input(event.assume_safe());
    }

    #[method]
    unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
        self.metascape_manager.draw(owner);
    }

    #[method]
    unsafe fn _process(&mut self, #[base] owner: &Node2D, delta: f32) {
        // Somehow delta can be negative...
        self.metascape_manager.process(owner, delta.clamp(0.0, 1.0));
    }

    #[method]
    unsafe fn get_debug_info(&mut self) -> String {
        self.metascape_manager.update_debug_info = true;
        std::mem::take(&mut self.metascape_manager.last_debug_info)
    }

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

// unsafe fn _update(client: &mut Client, owner: &Node2D, mut delta: f32) {
//     // Somehow delta can be negative...
//     delta = delta.clamp(0.0, 1.0);

//     client.player_inputs.update(owner);

//     match &mut client.client_state {
//         ClientState::Connected(client_metascape) => {
//             // Handle the signals from the client metascape.
//             for metascape_signal in client_metascape.update(delta, &client.player_inputs) {
//                 match metascape_signal {
//                     MetascapeSignal::Disconnected { reason } => {
//                         let reason_str = reason.to_string();
//                         log::info!("Disconnected: {}", &reason_str);
//                         owner.emit_signal("Disconnected", &[reason_str.to_variant()]);
//                     }
//                     MetascapeSignal::HasFleetChanged(has_fleet) => {
//                         log::info!("Has fleet changed: {}", has_fleet);
//                         owner.emit_signal("HasFleetChanged", &[has_fleet.to_variant()]);
//                     }
//                 }
//             }
//         }
//         ClientState::PendingConnection(pending_connection) => {
//             pending_connection.wait_duration += delta;

//             // Try get the login accepted packet.
//             let mut login_failed = false;
//             let mut client_metascape = None;
//             if let Some(connection) = &mut pending_connection.connection {
//                 let mut login_accepted: Option<LoginAccepted> = None;

//                 connection.recv_packets(|packet| match &packet {
//                     ServerPacket::Invalid | ServerPacket::DisconnectedReason(_) => {
//                         log::error!("{:?} while awaiting a login response.", packet);
//                         login_failed = true;
//                     }
//                     ServerPacket::ConnectionQueueLen(_) => {}
//                     ServerPacket::LoginResponse(response) => {
//                         if let LoginResponse::Accepted(response) = response {
//                             login_accepted = Some(response.to_owned());
//                         } else {
//                             log::error!("{:?} while awaiting a login response.", packet);
//                             login_failed = true;
//                         }
//                     }
//                     _ => pending_connection.unhandled_packets.push(packet),
//                 });

//                 if let Some(login_accepted) = login_accepted {
//                     // Create the client metascape.

//                     let connection = pending_connection.connection.take().unwrap();

//                     // TODO: Mods/data manager.
//                     let file = File::new();
//                     file.open(crate::constants::SYSTEMS_FILE_PATH, File::READ).unwrap();
//                     let buffer = file.get_buffer(file.get_len());
//                     file.close();
//                     let systems = bincode::deserialize::<metascape::Systems>(&buffer.read()).unwrap();

//                     client_metascape = Some(ClientMetascape::new(
//                         connection,
//                         login_accepted,
//                         ClientConfigs::default(),
//                         systems,
//                     ));
//                 }
//             } else {
//                 login_failed = true;
//             }

//             if login_failed {
//                 client.client_state = ClientState::Unconnected;
//                 log::warn!("Connection failed.");
//                 owner.emit_signal("ConnectionResult", &[false.to_variant()]);
//             } else if pending_connection.wait_duration > PendingConnection::MAX_WAIT_DURATION {
//                 client.client_state = ClientState::Unconnected;
//                 log::warn!("Connection attempt timedout.");
//                 owner.emit_signal("ConnectionResult", &[false.to_variant()]);
//             } else if let Some(client_metascape) = client_metascape {
//                 client.client_state = ClientState::Connected(client_metascape);
//                 log::info!("Connection successful");
//                 owner.emit_signal("ConnectionResult", &[true.to_variant()]);
//             }
//         }
//         ClientState::Unconnected => {}
//     }

//     owner.update();
// }

// /// Return if already connected or pending connection.
// unsafe fn _connect_local(client: &mut Client, client_id: u32) -> bool {
//     if let ClientState::Unconnected = &client.client_state {
//         let client_id = ClientId(client_id);

//         let (cc, mc) = OfflineConnectionClientSide::new();
//         let offline_connections_manager = OfflineConnectionsManager {
//             pending_connection: Some((Auth::Local(client_id), mc)),
//         };

//         client.client_state =
//             ClientState::PendingConnection(PendingConnection::new(ConnectionClientSideWrapper::Offline(cc)));

//         // TODO: Mods/data manager.
//         let file = File::new();
//         file.open(crate::constants::SYSTEMS_FILE_PATH, File::READ).unwrap();
//         let buffer = file.get_buffer(file.get_len());
//         file.close();
//         let systems = bincode::deserialize::<metascape::Systems>(&buffer.read()).unwrap();

//         let metascape = Metascape::new(Default::default(), systems.clone(), Default::default());
//         let metascape_manager = MetascapeManager::new(metascape, offline_connections_manager);

//         client.metascape_manager = Some(metascape_manager);
//     }

//     true
// }
