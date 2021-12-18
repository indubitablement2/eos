use crate::client::Client;
use crate::input_handler::InputHandler;
use crate::util::*;
use ahash::AHashMap;
use common::idx::*;
use common::packets::*;
use common::parameters::MetascapeParameters;
use common::system::Systems;
use common::UPDATE_INTERVAL;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;
use indexmap::IndexMap;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
enum MetascapeCommand {}

struct MetascapeEntityState {
    /// Used for fade in/out.
    fade: f32,
    previous_tick: u32,
    current_tick: u32,
    previous_position: Vec2,
    current_position: Vec2,
}
struct MetascapeFleet {
    fleet_id: FleetId,
}

struct MetascapeState {
    /// The current tick.
    tick: u32,
    /// How far are from previous to current tick.
    state_delta: f32,
    /// Multiply how fast tick increment.
    time_multiplier: f32,
    entity_state: IndexMap<u32, MetascapeEntityState>,
    entity_fleet: AHashMap<u32, MetascapeFleet>,
}
impl Default for MetascapeState {
    fn default() -> Self {
        Self {
            tick: 0,
            state_delta: 0.0,
            time_multiplier: 1.0,
            entity_state: IndexMap::new(),
            entity_fleet: AHashMap::new(),
        }
    }
}

pub struct ClientMetascape {
    metascape_parameters: MetascapeParameters,
    systems: Systems,

    /// Send input to server. Receive command from server.
    client: Client,
    send_timer: f32,

    recent_ping: VecDeque<f32>,
    average_ping: f32,

    metascape_state: MetascapeState,
    /// The expected entity order for a particular tick.
    /// Keys are order id.
    entity_orders: AHashMap<u8, (u32, Vec<u32>)>,

    metascape_data_commands: Vec<(u32, MetascapeCommand)>,
    state_buffer: Vec<MetascapeStatePart>,
}
impl ClientMetascape {
    pub fn new(
        server_addresses: ServerAddresses,
        metascape_parameters: MetascapeParameters,
    ) -> std::io::Result<Self> {
        Ok(Self {
            client: Client::new(server_addresses)?,
            systems: Systems(Vec::new()),
            metascape_parameters,
            state_buffer: Vec::new(),
            metascape_data_commands: Vec::new(),
            send_timer: 0.0,
            metascape_state: MetascapeState::default(),
            entity_orders: AHashMap::new(),
            recent_ping: VecDeque::new(),
            average_ping: 0.0,
        })
    }

    /// Return true if we should quit.
    pub fn update(&mut self, delta: f32, input_handler: &InputHandler) -> bool {
        let mut quit = false;

        // Handle pings.
        loop {
            match self.client.ping_duration_receiver.try_recv() {
                Ok(ping_time) => {
                    info!("{}", ping_time);
                    self.recent_ping.push_back(ping_time);
                    // Only keep 10 recent ping time.
                    while self.recent_ping.len() > 10 {
                        self.recent_ping.pop_front();
                    }
                }
                Err(err) => {
                    if err == crossbeam_channel::TryRecvError::Disconnected {
                        warn!("Ping loop disconnected. Quitting...");
                        quit = true;
                    }
                    break;
                }
            }
        }
        if let Some(total_ping) = self.recent_ping.iter().copied().reduce(|acc, dur| acc + dur) {
            self.average_ping = total_ping / self.recent_ping.len() as f32;
        }

        // Handle server tcp packets.
        loop {
            match self.client.tcp_receiver.try_recv() {
                Ok(tcp_packet) => match tcp_packet {
                    TcpServer::EntityList {
                        tick,
                        entity_order_id,
                        list,
                    } => {
                        self.entity_orders.insert(entity_order_id, (tick, list));
                        debug!("added an entity order.");
                        // Remove obselete entity order.
                        if self
                            .entity_orders
                            .remove(&entity_order_id.wrapping_add(u8::MAX / 2))
                            .is_some()
                        {
                            debug!("deleted an entity order.");
                        }
                    }
                    TcpServer::FleetInfo { entity_id, fleet_id } => {
                        self.metascape_state
                            .entity_fleet
                            .insert(entity_id, MetascapeFleet { fleet_id });
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
                    UdpServer::MetascapeEntityPosition(metascape_state_part) => {
                        // Add to state buffer.
                        self.state_buffer.push(metascape_state_part);
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

        // Get the last state we are ready to use.
        let max_tick = self.state_buffer.iter().fold(0, |acc, state| {
            // Check if we have the order for this state.
            if self.entity_orders.contains_key(&state.entity_order_required) {
                acc.max(state.tick)
            } else {
                acc
            }
        });
        let tick_delta = max_tick.saturating_sub(self.metascape_state.tick);
        if tick_delta > 10 {
            // We need to catch up.
            let previous_tick = self.metascape_state.tick;
            self.metascape_state.tick = max_tick.saturating_sub(4);
            self.metascape_state.state_delta = 0.0;
            debug!(
                "Client metascape state is behind by {}. Catching up from tick {} to {}...",
                tick_delta, previous_tick, self.metascape_state.tick
            );
        } else if tick_delta < 2 {
            // We need to keep a buffer.
            let previous_tick = self.metascape_state.tick;
            self.metascape_state.tick = max_tick.saturating_sub(5);
            self.metascape_state.state_delta = 0.0;
            if max_tick == 0 {
                debug!("buffering...");
            } else {
                debug!(
                    "Client metascape buffer is too small.
                Current tick: {},
                Current buffer: {},
                Most recent server tick: {},
                New current tick: {},
                New buffer: {},",
                    previous_tick,
                    tick_delta,
                    max_tick,
                    self.metascape_state.tick,
                    max_tick - self.metascape_state.tick,
                );
            }
        }

        // Speedup/slowdown time to get to target tick.
        let current_delta = tick_delta as f32 + self.metascape_state.state_delta;
        let target_delta = self.average_ping / UPDATE_INTERVAL.as_secs_f32() + 3.0;
        let mut target_time_multiplier = (current_delta / target_delta).clamp(0.66, 1.5);
        if (1.0 - target_time_multiplier).abs() < 1.2 {
            // When we are close to 1.0 multiplier, try not to fluctuate much.
            target_time_multiplier = target_time_multiplier.mul_add(0.25, 0.75);
        }
        self.metascape_state.time_multiplier *= 0.50;
        self.metascape_state.time_multiplier += target_time_multiplier * 0.2;
        self.metascape_state.time_multiplier += 0.30;
        self.metascape_state.state_delta +=
            (delta / UPDATE_INTERVAL.as_secs_f32()) * self.metascape_state.time_multiplier;
        if self.metascape_state.state_delta >= 1.0 {
            self.metascape_state.tick += 1;
            self.metascape_state.state_delta -= 1.0;
            self.metascape_state.state_delta = self.metascape_state.state_delta.clamp(-1.0, 1.0);
        }
        if (1.0 - self.metascape_state.time_multiplier).abs() > 1.1 {
            debug!("time_multiplier: {}", self.metascape_state.time_multiplier);
        }

        let current_tick = self.metascape_state.tick;

        // TODO: Remove obselete entities.

        // Consume states.
        for state in self.state_buffer.drain_filter(|state| state.tick <= current_tick) {
            let order = match self.entity_orders.get(&state.entity_order_required) {
                Some((_, o)) => o,
                None => {
                    warn!("No entity order for state at tick {}. Removing state...", state.tick);
                    continue;
                }
            };

            // Update each entity position.
            for (state_relative_pos, i) in state
                .entities_position
                .into_iter()
                .zip((state.part as usize * MetascapeStatePart::NUM_ENTITIES_POSITION_MAX)..)
            {
                let state_pos = state_relative_pos + state.relative_position;
                if let Some(entity_id) = order.get(i) {
                    if let Some(entity) = self.metascape_state.entity_state.get_mut(entity_id) {
                        if state.tick > entity.current_tick {
                            entity.previous_tick = entity.current_tick;
                            entity.previous_position = entity.current_position;
                            entity.current_tick = state.tick;
                            entity.current_position = state_pos;
                        } else if state.tick > entity.previous_tick {
                            entity.previous_tick = state.tick;
                            entity.previous_position = state_pos;
                        } else {
                            debug!(
                                "Received useless state. state: {} entity: {}. Ignoring...",
                                state.tick, entity.current_tick
                            );
                        }
                    } else {
                        // Create entity.
                        let new_entity = MetascapeEntityState {
                            fade: 0.0,
                            previous_tick: state.tick,
                            current_tick: state.tick,
                            previous_position: state_pos,
                            current_position: state_pos,
                        };
                        self.metascape_state.entity_state.insert(*entity_id, new_entity);
                    }
                } else {
                    warn!("Missing entity in order. Ignoring entity...");
                }
            }
        }

        // TODO: Consume MetascapeCommand.
        for (tick, c) in self
            .metascape_data_commands
            .drain_filter(|(tick, _)| *tick <= current_tick)
        {}

        // Send client packets to server.
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
        for (entity_id, entity) in self.metascape_state.entity_state.iter_mut() {
            if entity.current_tick < self.metascape_state.tick {
                entity.fade = (entity.fade - 0.05).max(0.0);
            } else {
                entity.fade = (entity.fade + 0.05).min(1.0);
            }
            if entity.fade <= 0.0 {
                continue;
            }

            // Interpolate position.
            let interpolation = (self.metascape_state.tick.saturating_sub(1 + entity.previous_tick)) as f32
                + self.metascape_state.state_delta;
            let pos = entity.previous_position.lerp(entity.current_position, interpolation);
            // let pos = entity.current_position;

            let mut r = 0.0;
            let mut g = 0.0;
            let b = (*entity_id % 10) as f32 / 10.0;
            let a = entity.fade * 0.9;

            if let Some(fleet_info) = self.metascape_state.entity_fleet.get(entity_id) {
                if ClientId::from(fleet_info.fleet_id) == self.client.client_id {
                    // This is us!
                    g = 1.0;
                    owner.draw_circle(
                        glam_to_godot(pos),
                        50.0,
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.2,
                        },
                    );
                }
            } else {
                // We don't know who is this entity.
                r = 1.0;
            }

            // Draw entity.
            owner.draw_circle(glam_to_godot(pos), 10.0, Color { r, g, b, a });
        }
    }
}
