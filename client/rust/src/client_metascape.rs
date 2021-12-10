use crate::client::Client;
use crate::input_handler::InputHandler;
use crate::util::*;
use common::packets::*;
use gdnative::api::*;
use gdnative::prelude::*;
use common::res_time::TimeRes;
use common::system::Systems;
use common::parameters::MetascapeParameters;
use common::generation::GenerationParameters;
use glam::Vec2;

pub struct ClientMetascape {
    /// Send input to server. Receive command from server.
    client: Client,
    time: TimeRes,
    metascape_parameters: MetascapeParameters,
    systems: Systems,
    detected_fleets: Vec<Vec2>,
}
impl ClientMetascape {
    pub fn new(
        server_addresses: ServerAddresses,
        metascape_parameters: MetascapeParameters,
        generation_parameters: GenerationParameters,
    ) -> std::io::Result<Self> {
        Ok(Self {
            client: Client::new(server_addresses)?,
            time: TimeRes::default(),
            systems: Systems::generate(&generation_parameters, &metascape_parameters),
            metascape_parameters,
            detected_fleets: Vec::new(),
            
        })
    }

    pub fn update(&mut self, input_handler: &InputHandler) {
        // Handle server packets.
        loop {
            match self.client.udp_receiver.try_recv() {
                Ok(udp_packet) => match udp_packet {
                    UdpServer::Battlescape { client_inputs, tick } => {

                    }
                    UdpServer::Metascape { fleets_position, tick } => {
                        self.detected_fleets = fleets_position;
                    }
                },
                Err(err) => {
                    if err == crossbeam_channel::TryRecvError::Disconnected {
                        error!("Client disconnected.");
                    }
                    break;
                }
            }
        }

        // Send client packets.
        // TODO: Only send packet every 100ms.
        let wish_position = input_handler.relative_mouse_position;
        let packet = UdpClient::Metascape {
            wish_position,
        };
        self.client.udp_sender.send(packet).unwrap();
    }

    pub fn render(&mut self, owner: &Node2D) {
        for position in &self.detected_fleets {
            // Draw all fleet.
            owner.draw_circle(
                glam_to_godot(*position),
                10.0,
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 0.5,
                },
            );
        }
    }
}