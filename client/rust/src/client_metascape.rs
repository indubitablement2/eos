use crate::client::Client;
use crate::input_handler::InputHandler;
use crate::util::*;
use ahash::AHashMap;
use ahash::AHashSet;
use common::UPDATE_INTERVAL;
use common::generation::GenerationParameters;
use common::idx::*;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::system::Systems;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;
use indexmap::IndexSet;

#[derive(Debug, Clone)]
enum MetascapeDataCommand {
}
impl MetascapeDataCommand {
    fn apply(&self, mut metascape_data: &mut MetascapeData) {
    }
}

#[derive(Debug, Clone)]
struct MetascapeData {
    /// Ordered the same as the server would send entity position.
    entity_order: IndexSet<u64>,
}
impl Default for MetascapeData {
    fn default() -> Self {
        Self { entity_order: IndexSet::new() }
    }
}

struct MetascapeState {
    tick: u64,
    entities_position: Vec<Vec2>,
}
impl Default for MetascapeState {
    fn default() -> Self {
        Self {
            tick: 0,
            entities_position: Vec::new(),
        }
    }
}

struct IncompleteState {
    tick: u64,
    parts: AHashMap<u8, MetascapeState>,
}

pub struct ClientMetascape {
    metascape_parameters: MetascapeParameters,
    systems: Systems,

    /// Send input to server. Receive command from server.
    client: Client,
    send_timer: f32,

    previous_metascape_data: MetascapeData,
    current_metascape_data: MetascapeData,
    metascape_order_update: Vec<(u64, Vec<u64>)>,
    metascape_data_commands: Vec<(u64, MetascapeDataCommand)>,

    /// How far are we from previous state to current state.
    state_delta: f32,

    previous_state: MetascapeState,
    current_state: MetascapeState,
    state_buffer: Vec<MetascapeState>,
    incomplete_states: Vec<IncompleteState>,
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
            previous_metascape_data: MetascapeData::default(),
            current_metascape_data: MetascapeData::default(),
            metascape_data_commands: Vec::new(),
            incomplete_states: Vec::new(),
            send_timer: 0.0,
            metascape_order_update: Vec::new(),
        })
    }

    /// Return true if we should quit.
    pub fn update(&mut self, delta: f32, input_handler: &InputHandler) -> bool {
        let mut quit = false;

        self.state_delta += delta / UPDATE_INTERVAL.as_secs_f32();

        let mut previous_tick = self.previous_state.tick;
        let mut current_tick = self.current_state.tick;

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
                        if metascape_tick < current_tick {
                            // This state is too old to be useful.
                            continue;
                        }

                        // Add to state buffer.
                        self.state_buffer.push(MetascapeState {
                            tick: metascape_tick,
                            entities_position,
                        });
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

        // Handle server tcp packets.
        loop {
            match self.client.tcp_receiver.try_recv() {
                Ok(tcp_packet) => {
                    match tcp_packet {
                        TcpServer::EntityList { tick, list} => {
                            self.metascape_order_update.push((tick, list));
                        }
                    }
                }
                Err(err) => {
                    if err == crossbeam_channel::TryRecvError::Disconnected {
                        warn!("No tcp connection to the server. Quitting...");
                        quit = true;
                    }
                    break;
                }
            }
        }

        // Sort metascape_data_commands by tick.
        self.metascape_data_commands.sort_by(|(a, _), (b, _)| {
            a.cmp(b)
        });

        // Get most sensible tick.
        if let Some(last_received_state) =  self.state_buffer.last() {
            if last_received_state.tick - current_tick > 3 {
                debug!("Client metascape state is behind. Catching up...");
                let target_tick = last_received_state.tick.saturating_sub(2);
                for state in self.state_buffer.iter().rev() {
                    if state.tick <= target_tick {
                        current_tick = state.tick;
                        break;
                    }
                }
                debug!("New tick {}.\nTarget tick: {}\nMost recent tick available: {}", current_tick, target_tick, last_received_state.tick);
                self.state_delta = 0.0;
            }
        }
        if self.state_delta >= 1.0 {
            // Get the next available tick.
            let target_tick = current_tick + 1;
            for state in self.state_buffer.iter() {
                current_tick = state.tick;
                if current_tick >= target_tick {
                    break;
                }
            }
        }

        // Update current metascape state.
        for state in self.state_buffer.drain_filter(|state| state.tick <= current_tick) {
            if state.tick == current_tick {
                // Previous state takes current state.
                std::mem::swap(&mut self.previous_state, &mut self.current_state);
                self.previous_metascape_data = self.current_metascape_data.clone();

                // Update interpolation delta.
                let new_previous_tick = self.previous_state.tick;
                self.state_delta -= (new_previous_tick - previous_tick) as f32;
                self.state_delta = self.state_delta.max(0.0);
                previous_tick = new_previous_tick;
                
                self.current_state = state;
            }
        }

        // Update entity order.
        for (tick, new_order) in self.metascape_order_update.drain_filter(|(tick, _)| {
            *tick <= current_tick
        }) {
            if tick <= previous_tick {
                self.previous_metascape_data.entity_order = new_order.into_iter().collect();
            } else if tick <= current_tick {
                self.current_metascape_data.entity_order = new_order.into_iter().collect();
            }
        }

        // TODO: Consume MetascapeDataCommand.
        for (tick , c) in self.metascape_data_commands.drain_filter(|(tick, _)| *tick <= current_tick) {
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
        let mut pre_iter = self.previous_metascape_data.entity_order.iter().peekable();
        let mut cur_iter = self.current_metascape_data.entity_order.iter().peekable();
        
        let mut pre_pos_iter = self.previous_state.entities_position.iter();
        let mut cur_pos_iter = self.current_state.entities_position.iter();

        loop {
            let mut entity_is_previous = false;
            let mut entity_is_current = false;
            let entity_id = if let Some(c) = cur_iter.peek().copied().copied() {
                if let Some(p) = pre_iter.peek().copied().copied() {
                    if p == c {
                        cur_iter.next();
                        pre_iter.next();
                        entity_is_previous = true;
                        entity_is_current = true;
                        p
                    } else if p < c {
                        pre_iter.next();
                        entity_is_previous = true;
                        p
                    } else {
                        cur_iter.next();
                        entity_is_current = true;
                        c
                    }
                } else {
                    cur_iter.next();
                    entity_is_current = true;
                    c
                }
            } else {
                if let Some(p) = pre_iter.peek().copied().copied() {
                    pre_iter.next();
                    entity_is_previous = true;
                    p
                } else {
                    // No more entity.
                    break;
                }
            };

            let mut r = 0.0;
            let g = 0.0;
            let b = (entity_id % 10) as f32 / 10.0;
            let mut a = 1.0;
            let pos = if entity_is_previous && entity_is_current {
                // We ca interpolate this entity normaly.
                let from_pos = pre_pos_iter.next().unwrap_or_else(|| {
                    debug!("Previous state is missing an entity.");
                    r = 1.0;
                    &Vec2::ZERO
                });
                let to_pos = cur_pos_iter.next().unwrap_or_else(|| {
                    debug!("Current state is missing an entity.");
                    r = 1.0;
                    from_pos
                });
                from_pos.lerp(*to_pos, self.state_delta)
            } else if entity_is_current {
                // Entity is new. Fade in.
                a = self.state_delta;

                *cur_pos_iter.next().unwrap_or_else(|| {
                    debug!("Current state is missing an entity.");
                    r = 1.0;
                    &Vec2::ZERO
                })
            } else {
                // Entity is gone. Fade out.
                a -= self.state_delta;

                *pre_pos_iter.next().unwrap_or_else(|| {
                    debug!("Previous state is missing an entity.");
                    r = 1.0;
                    &Vec2::ZERO
                })
            };

            // Draw all entity.
            owner.draw_circle(
                glam_to_godot(pos),
                10.0,
                Color {
                    r,
                    g,
                    b,
                    a,
                },
            );
        }
    }
}
