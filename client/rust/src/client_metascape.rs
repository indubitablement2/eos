use crate::configs::Configs;
use crate::connection_manager::ConnectionManager;
use crate::constants::GAME_TO_GODOT_RATIO;
use crate::constants::WORLD_DATA_FILE_PATH;
use crate::input_handler::PlayerInputs;
use crate::util::*;
use ahash::AHashMap;
use common::idx::*;
use common::intersection::*;
use common::orbit::Orbit;
use common::packets::*;
use common::tcp_loops::TcpOutboundEvent;
use common::world_data::WorldData;
use common::UPDATE_INTERVAL;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

#[derive(Debug, Clone, Copy, Default)]
struct EntityState {
    /// The tick we first learned about this entity.
    discovered_tick: u32,
    previous_tick: u32,
    current_tick: u32,
    previous_position: Vec2,
    current_position: Vec2,
    /// The tick the orbit was added.
    /// The entity currently has an orbit if this is more than `current_tick`.
    orbit_added_tick: u32,
    orbit: Orbit,
}
impl EntityState {
    pub fn get_interpolated_pos(&self, time: f32) -> Vec2 {
        if self.orbit_added_tick >= self.current_tick {
            self.orbit.to_position(time)
        } else {
            let interpolation = time - 1.0 - self.previous_tick as f32;
            self.previous_position.lerp(self.current_position, interpolation)
        }
    }

    fn update(&mut self, new_tick: u32, new_position: Vec2) {
        if new_tick > self.current_tick {
            self.previous_tick = self.current_tick;
            self.previous_position = self.current_position;
            self.current_tick = new_tick;
            self.current_position = new_position;
        } else if new_tick > self.previous_tick {
            self.previous_tick = new_tick;
            self.previous_position = new_position;
        } else {
            debug!(
                "Received useless state. state: {} entity: {}. Ignoring...",
                new_tick, self.current_tick
            );
        }
    }
}

pub struct Metascape {
    configs: Configs,

    world_data: WorldData,
    systems_acceleration: AccelerationStructure,

    /// Send input to server. Receive command from server.
    connection_manager: ConnectionManager,
    send_timer: f32,

    /// The current tick.
    tick: u32,
    /// How far are from current to next tick.
    delta: f32,
    /// Multiply how fast tick increment.
    time_multiplier: f32,
    /// The last tick received from the server.
    max_tick: u32,

    client_state: EntityState,
    entities_state: AHashMap<u16, EntityState>,
    entities_state_buffer: Vec<EntitiesState>,

    client_info: EntityInfo,
    entities_info: AHashMap<u16, EntityInfo>,
    entities_info_buffer: Vec<EntitiesInfo>,
    entities_remove_buffer: Vec<EntitiesRemove>,
}
impl Metascape {
    pub fn new(connection_manager: ConnectionManager, configs: Configs) -> std::io::Result<Self> {
        let client_id = connection_manager.client_id;

        // Load world data from file.
        let file = File::new();
        if let Err(err) = file.open(WORLD_DATA_FILE_PATH, File::READ) {
            error!("{:?} can not open ({})", err, WORLD_DATA_FILE_PATH);
            file.close();
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Can not open file."));
        }
        let buffer = file.get_buffer(file.get_len());
        file.close();
        let world_data: WorldData = if let Ok(world_data) = bincode::deserialize(&buffer.read()) {
            world_data
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Can not deserialize world data file.",
            ));
        };

        // Create acceleration structure.
        let mut systems_acceleration = AccelerationStructure::new();
        systems_acceleration.colliders.extend(
            world_data
                .systems
                .iter()
                .map(|(id, system)| Collider::new(id.0, system.bound, system.position)),
        );
        systems_acceleration.update();

        Ok(Self {
            configs,
            world_data,
            systems_acceleration,
            connection_manager,
            send_timer: 0.0,
            tick: 0,
            delta: 0.0,
            time_multiplier: 1.0,
            client_state: EntityState::default(),
            entities_state: AHashMap::new(),
            entities_info_buffer: Vec::new(),
            entities_state_buffer: Vec::new(),
            max_tick: 0,
            client_info: EntityInfo {
                info_type: EntityInfoType::Fleet(FleetInfo {
                    fleet_id: FleetId::from(client_id),
                    composition: Vec::new(),
                }),
                name: String::new(),
                orbit: None,
            },
            entities_info: AHashMap::new(),
            entities_remove_buffer: Vec::new(),
        })
    }

    /// Return true if we should quit.
    pub fn update(&mut self, delta: f32, player_inputs: &PlayerInputs) -> bool {
        let mut quit = false;

        // Handle server packets.
        loop {
            match self.connection_manager.inbound_receiver.try_recv() {
                Ok(packet) => match Packet::deserialize(&packet) {
                    Packet::BattlescapeCommands(new_commands) => todo!(),
                    Packet::EntitiesState(new_states) => {
                        self.max_tick = self.max_tick.max(new_states.tick);

                        self.entities_state_buffer.push(new_states);
                    }
                    Packet::EntitiesInfo(new_infos) => {
                        self.entities_info_buffer.push(new_infos);
                    }
                    Packet::DisconnectedReason(reason) => {
                        debug!("Disconnected from the server. {}", reason);
                        // TODO: Send message to console.
                        quit = true;
                        break;
                    }
                    Packet::EntitiesRemove(new_remove) => {
                        self.entities_remove_buffer.push(new_remove);
                    }
                    Packet::Message { origin, content } => todo!(),
                    _ => {
                        warn!("Received an invalid packet from the server. Quitting...");
                        quit = true;
                        break;
                    }
                },
                Err(err) => {
                    if err.is_disconnected() {
                        warn!("Disconnected from the server. Quitting...");
                        quit = true;
                    }
                    break;
                }
            }
        }

        // Hard catch up if we are too beind in tick.
        let tick_delta = self.max_tick.saturating_sub(self.tick);
        if tick_delta > 10 {
            let previous_tick = self.tick;
            self.tick = self.max_tick.saturating_sub(5);
            self.delta = 0.0;
            debug!(
                "Client metascape state is behind by {}. Catching up from tick {} to {}...",
                tick_delta, previous_tick, self.tick
            );
        }

        // Speedup/slowdown time to get to target tick.
        let current_delta = tick_delta as f32 + self.delta;
        let mut target_time_multiplier = (current_delta / self.configs.target_delta).clamp(0.66, 1.5);
        if (current_delta - self.configs.target_delta).abs() < 1.5 {
            // When we are close to the target, try not to fluctuate much.
            target_time_multiplier = target_time_multiplier.mul_add(0.1, 0.9);
        }
        self.time_multiplier *= 0.50;
        self.time_multiplier += target_time_multiplier * 0.2;
        self.time_multiplier += 0.30;
        self.delta += (delta / UPDATE_INTERVAL.as_secs_f32()) * self.time_multiplier;
        if self.time_multiplier > 1.1 || self.time_multiplier < 0.95 {
            debug!("time_multiplier: {}", self.time_multiplier);
        }

        // Increment tick.
        while self.delta >= 1.0 {
            self.tick += 1;
            self.delta -= 1.0;
        }

        let current_tick = self.tick;

        // Remove obselete entities.
        for remove_update in self
            .entities_remove_buffer
            .drain_filter(|update| update.tick <= current_tick)
        {
            for id in remove_update.to_remove.into_iter() {
                if self.entities_info.remove(&id).is_none() || self.entities_state.remove(&id).is_none() {
                    warn!("Got order to remove {}, but it is not added. Ignoring...", id);
                }
            }
        }

        // Consume infos.
        for infos_update in self.entities_info_buffer.drain_filter(|info| info.tick <= current_tick) {
            // Handle client info update.
            if let Some(info) = infos_update.client_info {
                if let Some(orbit) = info.orbit {
                    self.client_state.orbit = orbit;
                    self.client_state.orbit_added_tick = infos_update.tick;
                }
                self.client_info = info;
            }
            // Handle entities info update.
            for (id, info) in infos_update.infos.into_iter() {
                if !self.entities_state.contains_key(&id) {
                    self.entities_state.insert(id, EntityState::default());
                }
                if let Some(orbit) = info.orbit {
                    if let Some(state) = self.entities_state.get_mut(&id) {
                        state.orbit = orbit;
                        state.orbit_added_tick = infos_update.tick;
                    }
                }
                self.entities_info.insert(id, info);
            }
        }

        // Consume states.
        for state in self
            .entities_state_buffer
            .drain_filter(|state| state.tick <= current_tick)
        {
            // Update the client state.
            self.client_state.update(state.tick, state.client_entity_position);

            // Update each entities position.
            for (id, mut position) in state.relative_entities_position.into_iter() {
                if let Some(entity) = self.entities_state.get_mut(&id) {
                    // Convert to world position.
                    position += state.client_entity_position;
                    entity.update(state.tick, position);
                } else {
                    // Create new entity.
                    debug!("missing entity.");
                }
            }
        }

        // Handle client inputs.
        if player_inputs.primary {
            let packet = Packet::MetascapeWishPos {
                wish_pos: player_inputs.global_mouse_position,
            }
            .serialize();
            quit |= self
                .connection_manager
                .tcp_outbound_event_sender
                .blocking_send(TcpOutboundEvent::PacketEvent(packet))
                .is_err();
        }

        // Send client packets to server.
        self.send_timer -= delta;
        if self.send_timer <= 0.0 {
            // TODO: Maybe send more often?
            self.send_timer = UPDATE_INTERVAL.as_secs_f32();
            quit |= self
                .connection_manager
                .tcp_outbound_event_sender
                .blocking_send(TcpOutboundEvent::FlushEvent)
                .is_err();
        }

        quit
    }

    pub fn render(&mut self, owner: &Node2D) {
        let time = self.tick as f32 + self.delta;

        // Debug draw entities.
        for (id, entity) in self.entities_state.iter() {
            error!("there should be none!");
            let fade = ((self.tick as f32 - entity.discovered_tick as f32) * 0.1).clamp(0.1, 1.0);

            // Interpolate position.
            let pos = entity.get_interpolated_pos(time);

            let mut r = 0.0;
            let g = 0.0;
            let b = (*id % 10) as f32 / 10.0;
            let a = fade * 0.8;

            if let Some(info) = self.entities_info.get(id) {
                // TODO: Do something with the fleet info.
            } else {
                // We don't know who is this entity.
                r = 1.0;
            }

            // Draw entity.
            owner.draw_circle(pos.to_godot_scaled(), 8.0, Color { r, g, b, a });
        }

        // Debug draw our entity.
        let pos = self.client_state.get_interpolated_pos(time);
        if self.tick % 5 == 0 {
            debug!("{:?}", self.client_state.orbit);
        }
        owner.draw_circle(
            pos.to_godot_scaled(),
            8.0,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.8,
            },
        );

        // Debug draw systems.
        let screen_collider = Collider::new_idless(self.configs.system_draw_distance, pos);
        for system_id in self
            .systems_acceleration
            .intersect_collider(screen_collider)
            .into_iter()
            .map(|id| SystemId(id))
        {
            if let Some(system) = self.world_data.systems.get(&system_id) {
                for body in system.bodies.iter() {
                    let (r, g, b) = match body.body_type {
                        common::world_data::CelestialBodyType::Star => (1.0, 0.2, 0.0),
                        common::world_data::CelestialBodyType::Planet => (0.0, 0.5, 1.0),
                        common::world_data::CelestialBodyType::BlackHole => (0.0, 0.0, 0.0),
                    };

                    owner.draw_circle(
                        body.orbit.to_position(time).to_godot_scaled(),
                        (body.radius * GAME_TO_GODOT_RATIO).into(),
                        Color { r, g, b, a: 0.8 },
                    );
                }
            } else {
                warn!("Can not find system {:?}. Ignoring...", system_id);
            }
        }
    }
}
