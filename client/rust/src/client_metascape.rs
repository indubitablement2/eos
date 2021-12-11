use crate::client::Client;
use crate::input_handler::InputHandler;
use crate::util::*;
use common::generation::GenerationParameters;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::system::Systems;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

pub struct MetascapeState {
    tick: u64,
    fleets_position: Vec<Vec2>,
}
impl Default for MetascapeState {
    fn default() -> Self {
        Self {
            tick: 0,
            fleets_position: Vec::new(),
        }
    }
}

pub struct ClientMetascape {
    /// Send input to server. Receive command from server.
    client: Client,
    metascape_parameters: MetascapeParameters,
    systems: Systems,
    /// Detected fleet.
    previous_state: MetascapeState,
    /// Used with previous_fleets_position for interpolation.
    current_state: MetascapeState,
    /// Previously received packet that are not yet used.
    state_buffer: Vec<MetascapeState>,
    /// How far are we from previous state to current state.
    state_delta: f64,
}
impl ClientMetascape {
    pub fn new(
        server_addresses: ServerAddresses,
        metascape_parameters: MetascapeParameters,
        generation_parameters: GenerationParameters,
    ) -> std::io::Result<Self> {
        Ok(Self {
            client: Client::new(server_addresses)?,
            systems: Systems::generate(&generation_parameters, &metascape_parameters),
            metascape_parameters,
            previous_state: MetascapeState::default(),
            current_state: MetascapeState::default(),
            state_buffer: Vec::new(),
            state_delta: 0.0,
        })
    }

    pub fn update(&mut self, delta: f64, input_handler: &InputHandler) {
        // Handle server packets.
        loop {
            match self.client.udp_receiver.try_recv() {
                Ok(udp_packet) => match udp_packet {
                    UdpServer::Battlescape {
                        client_inputs,
                        battlescape_tick,
                    } => {}
                    UdpServer::Metascape {
                        fleets_position,
                        metascape_tick,
                    } => {
                        // Pack into a MetascapeState.
                        self.state_buffer.push(MetascapeState {
                            tick: metascape_tick,
                            fleets_position,
                        });
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
        let packet = UdpClient::Metascape { wish_position };
        self.client.udp_sender.send(packet).unwrap();
    }

    pub fn render(&mut self, owner: &Node2D) {
        // for position in &self.fleets_position {
        //     // Draw all fleet.
        //     owner.draw_circle(
        //         glam_to_godot(*position),
        //         10.0,
        //         Color {
        //             r: 0.0,
        //             g: 0.0,
        //             b: 1.0,
        //             a: 0.5,
        //         },
        //     );
        // }
    }
}
