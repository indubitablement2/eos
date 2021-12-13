use crate::client::Client;
use crate::input_handler::InputHandler;
use crate::util::*;
use ahash::AHashMap;
use common::generation::GenerationParameters;
use common::idx::*;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::system::Systems;
use common::UPDATE_INTERVAL;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;
use indexmap::IndexMap;
use indexmap::IndexSet;

#[derive(Debug, Clone)]
enum MetascapeDataCommand {}
impl MetascapeDataCommand {
    fn apply(&self, mut metascape_data: &mut MetascapeData) {}
}

#[derive(Debug, Clone)]
struct MetascapeData {
    /// Ordered the same as the server would send entity position.
    entity_order: IndexSet<u64>,
}
impl Default for MetascapeData {
    fn default() -> Self {
        Self {
            entity_order: IndexSet::new(),
        }
    }
}

struct MetascapeEntity {
    /// Used for fade in/out.
    fade: f32,
    previous_tick: u64,
    current_tick: u64,
    previous_position: Vec2,
    current_position: Vec2,
}

struct MetascapeState {
    /// The current tick.
    tick: u64,
    /// How far are from previous to current tick.
    state_delta: f32,
    entity: IndexMap<u64, MetascapeEntity>,
}
impl Default for MetascapeState {
    fn default() -> Self {
        Self {
            tick: 0,
            state_delta: 0.0,
            entity: IndexMap::new(),
        }
    }
}


struct IncompleteState {
    tick: u64,
    part: u8,
    entities_position: Vec<Vec2>,
}

pub struct ClientMetascape {
    metascape_parameters: MetascapeParameters,
    systems: Systems,

    /// Send input to server. Receive command from server.
    client: Client,
    send_timer: f32,

    metascape_state: MetascapeState,
    /// The expected entity order for a particular tick.
    entity_orders: Vec<(u64, Vec<u64>)>,

    metascape_data_commands: Vec<(u64, MetascapeDataCommand)>,

    last_received_state_tick: u64,
    state_buffer: Vec<IncompleteState>,
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
            state_buffer: Vec::new(),
            metascape_data_commands: Vec::new(),
            send_timer: 0.0,
            metascape_state: MetascapeState::default(),
            entity_orders: Vec::new(),
            last_received_state_tick: 0,
        })
    }

    /// Return true if we should quit.
    pub fn update(&mut self, delta: f32, input_handler: &InputHandler) -> bool {
        let mut quit = false;

        self.metascape_state.state_delta += delta / UPDATE_INTERVAL.as_secs_f32();

        // Handle server tcp packets.
        loop {
            match self.client.tcp_receiver.try_recv() {
                Ok(tcp_packet) => match tcp_packet {
                    TcpServer::EntityList { tick, list } => {
                        self.entity_orders.push((tick, list));
                    }
                },
                Err(err) => {
                    if err == crossbeam_channel::TryRecvError::Disconnected {
                        warn!("No tcp connection to the server. Quitting...");
                        quit = true;
                    }
                    break;
                }
            }
        }

        // Handle server udp packets.
        loop {
            match self.client.udp_receiver.try_recv() {
                Ok(udp_packet) => match udp_packet {
                    UdpServer::Battlescape {
                        client_inputs,
                        battlescape_tick,
                    } => {}
                    UdpServer::Metascape {
                        entities_position,
                        metascape_tick,
                    } => {
                        // Add to state buffer.
                        self.state_buffer.push(IncompleteState {
                            tick: metascape_tick,
                            part: 0,
                            entities_position,
                        });
                        self.last_received_state_tick = self.last_received_state_tick.max(metascape_tick);
                    }
                },
                Err(err) => {
                    if err == crossbeam_channel::TryRecvError::Disconnected {
                        warn!("No udp connection to the server. Quitting...");
                        quit = true;
                    }
                    break;
                }
            }
        }

        // Get most sensible tick.
        if self.last_received_state_tick - self.metascape_state.tick > 4 {
            self.metascape_state.tick = self.last_received_state_tick.saturating_sub(2);
            self.metascape_state.state_delta = 0.0;
            debug!(
                "Client metascape state is behind. Catching up to tick {}...",
                self.metascape_state.tick
            );
        }
        if self.metascape_state.state_delta >= 1.0 {
            self.metascape_state.tick += 1;
        }

        let current_tick = self.metascape_state.tick;

        // Consume states.
        // TODO: break that into multiple parts.
        for state in self.state_buffer.drain_filter(|state| state.tick <= current_tick) {
            for (order_tick, order) in self.entity_orders.iter().rev() {
                if *order_tick <= state.tick {
                    for (pos, i) in state.entities_position.into_iter().zip(state.part as usize * 25..) {
                        if let Some(entity_id) = order.get(i) {
                            if let Some(entity) = self.metascape_state.entity.get_mut(entity_id) {
                                if state.tick >= entity.current_tick {
                                    entity.previous_tick = entity.current_tick;
                                    entity.previous_position = entity.current_position;
                                    entity.current_tick = state.tick;
                                    entity.current_position = pos;
                                } else if state.tick >= entity.previous_tick {
                                    entity.previous_tick = state.tick;
                                    entity.previous_position = pos;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }

        // TODO: Consume MetascapeDataCommand.
        for (tick, c) in self
            .metascape_data_commands
            .drain_filter(|(tick, _)| *tick <= current_tick)
        {}

        // Remove obselete entity order.
        let mut num_old_order = 0usize;
        for (tick, _) in self.entity_orders.iter() {
            if *tick >= current_tick {
                break;
            }
            num_old_order += 1;
        }
        while num_old_order > 10 {
            self.entity_orders.remove(0);
            num_old_order -= 1;
        }

        // Send client packets.
        self.send_timer -= delta;
        if self.send_timer <= 0.0 {
            self.send_timer = UPDATE_INTERVAL.as_secs_f32();

            let wish_position = input_handler.relative_mouse_position;
            let packet = UdpClient::Metascape { wish_position };
            if self.client.udp_sender.send(packet).is_err() {
                warn!("No udp connection to the server. Quitting...");
                quit = true;
            }
        }

        quit
    }

    pub fn render(&mut self, owner: &Node2D) {
        for (entity_id, entity) in self.metascape_state.entity.iter() {
            let r = 0.0;
            let g = 0.0;
            let b = (*entity_id % 10) as f32 / 10.0;
            let a = entity.fade * 0.8;

            let interpolation =
                (self.metascape_state.tick - 1 - entity.previous_tick) as f32 + self.metascape_state.state_delta;

            // Interpolate position.
            let pos = entity.previous_position.lerp(entity.current_position, interpolation);

            // Draw entity.
            owner.draw_circle(glam_to_godot(pos), 10.0, Color { r, g, b, a });
        }
    }
}
