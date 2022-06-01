use crate::configs::Configs;
use crate::connection_manager::ConnectionManager;
use crate::constants::*;
use crate::input_handler::PlayerInputs;
use crate::util::*;
use ::utils::acc::*;
use ahash::AHashMap;
use common::idx::*;
use common::net::packets::*;
use common::system::*;
use common::UPDATE_INTERVAL;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

#[derive(Debug, Clone)]
struct FleetState {
    /// The tick we first learned about this entity.
    discovered_tick: u32,
    previous_tick: u32,
    current_tick: u32,
    previous_position: Vec2,
    current_position: Vec2,
    fleet_infos: FleetInfos,
}
impl FleetState {
    pub fn new(discovered_tick: u32) -> Self {
        Self {
            discovered_tick,
            previous_tick: 0,
            current_tick: 0,
            previous_position: Vec2::ZERO,
            current_position: Vec2::ZERO,
            fleet_infos: FleetInfos {
                fleet_id: FleetId(0),
                small_id: 0,
                name: "".to_string(),
                orbit: None,
                composition: Vec::new(),
            },
        }
    }

    pub fn get_interpolated_pos(&self, timef: f32) -> Vec2 {
        if let Some(orbit) = self.fleet_infos.orbit {
            orbit.to_position(timef)
        } else {
            let interpolation = timef - 1.0 - self.previous_tick as f32;
            self.previous_position.lerp(self.current_position, interpolation)
        }
    }

    fn update(&mut self, new_tick: u32, new_position: Vec2) {
        if self.current_tick == 0 {
            // Fresh fleet.
            self.current_tick = new_tick;
            self.current_position = new_position;
            self.previous_tick = new_tick;
            self.previous_position = new_position;
        } else if new_tick > self.current_tick {
            self.previous_tick = self.current_tick;
            self.previous_position = self.current_position;
            self.current_tick = new_tick;
            self.current_position = new_position;
        } else if new_tick > self.previous_tick {
            self.previous_tick = new_tick;
            self.previous_position = new_position;
        } else {
            debug!(
                "Received useless state. received state tick: {} current tick: {}. Ignoring...",
                new_tick, self.current_tick
            );
        }
    }
}

pub struct Metascape {
    configs: Configs,
    systems: Systems,
    systems_acceleration: AccelerationStructure<SystemId, ()>,

    /// Send input to server. Receive command from server.
    connection_manager: ConnectionManager,
    send_timer: f32,

    /// The current tick.
    tick: u32,
    /// How far are from current to next tick.
    delta: f32,
    /// How much time dilation we have applied last update.
    pub time_dilation: f32,
    /// Last frame target buffer.
    pub target_tick_buffer: u32,
    /// Last frame tick buffer.
    pub current_tick_buffer: i64,
    /// The last tick received from the server.
    max_tick: u32,
    /// The minimum tick delta of the 10 previous tick.
    min_buffer_short: [i64; 10],
    /// The minimum of the 30 previous `min_buffer_short`.
    min_buffer_long: [i64; 30],

    /// Client has `small_id` 0.
    fleets_state: AHashMap<u16, FleetState>,
    fleets_position_buffer: Vec<FleetsPosition>,
    fleets_infos_buffer: Vec<FleetsInfos>,
    fleets_forget_buffer: Vec<FleetsForget>,
}
impl Metascape {
    pub fn new(connection_manager: ConnectionManager, configs: Configs) -> anyhow::Result<Self> {
        // Load systems from file.
        let file = File::new();
        file.open(SYSTEMS_FILE_PATH, File::READ)?;
        let buffer = file.get_buffer(file.get_len());
        file.close();
        let systems = bincode::deserialize::<Systems>(&buffer.read())?;

        Ok(Self {
            configs,
            systems_acceleration: systems.create_acceleration_structure(),
            systems,
            connection_manager,
            send_timer: 0.0,
            tick: 0,
            delta: 0.0,
            time_dilation: 1.0,
            max_tick: 0,
            target_tick_buffer: 1,
            min_buffer_short: [1; 10],
            min_buffer_long: [1; 30],
            current_tick_buffer: 1,
            fleets_state: Default::default(),
            fleets_position_buffer: Default::default(),
            fleets_infos_buffer: Default::default(),
            fleets_forget_buffer: Default::default(),
        })
    }

    /// Return true if we should quit.
    pub fn update(&mut self, delta: f32, player_inputs: &PlayerInputs) -> bool {
        let mut quit = false;

        // Handle server packets.
        loop {
            match self.connection_manager.try_recv() {
                Ok(packet) => match ServerPacket::deserialize(&packet) {
                    ServerPacket::Invalid => {
                        warn!("Received an invalid packet from the server. Quitting...");
                        quit = true;
                        break;
                    }
                    ServerPacket::DisconnectedReason(reason) => {
                        debug!("Disconnected from the server. {}", reason);
                        // TODO: Send message to console.
                        quit = true;
                        break;
                    }
                    ServerPacket::ConnectionQueueLen(_) => todo!(),
                    ServerPacket::FleetsInfos(fleets_infos) => {
                        self.fleets_infos_buffer.push(fleets_infos);
                    }
                    ServerPacket::FleetsPosition(fleet_position) => {
                        self.max_tick = self.max_tick.max(fleet_position.tick);
                        self.fleets_position_buffer.push(fleet_position);
                    }
                    ServerPacket::FleetsForget(fleets_forget) => {
                        self.fleets_forget_buffer.push(fleets_forget);
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

        // Hard catch up if we are too out of sync in tick.
        let tick_delta = self.max_tick as i64 - self.tick as i64;
        if tick_delta > 10 {
            let previous_tick = self.tick;
            self.tick = self.max_tick.saturating_sub(5);
            self.delta = 0.0;
            debug!(
                "Client metascape state is behind by {}. Catching up from tick {} to {}...",
                tick_delta, previous_tick, self.tick
            );
        } else if tick_delta < -1 {
            let previous_tick = self.tick;
            self.tick = self.max_tick;
            self.delta = 0.0;
            debug!(
                "Client metascape state is forward by {}. Catching up from tick {} to {}...",
                tick_delta, previous_tick, self.tick
            );
        }

        // Compute target tick buffer. (how large we want the tick buffer)
        self.current_tick_buffer = self.max_tick as i64 - self.tick as i64;
        let i = self.tick as usize % self.min_buffer_short.len();
        self.min_buffer_short[i] = self.min_buffer_short[i].min(self.current_tick_buffer);
        let short_reduce = *self.min_buffer_short.iter().reduce(|acc, x| acc.min(x)).unwrap();
        let j = (self.tick as usize / self.min_buffer_short.len()) % self.min_buffer_long.len();
        self.min_buffer_long[j] = short_reduce;
        let long_reduce = *self.min_buffer_long.iter().reduce(|acc, x| acc.min(x)).unwrap();
        self.target_tick_buffer = long_reduce.min(0).abs() as u32 + 1; // The negative part + 1.

        let delta_target_tick_buffer = self.current_tick_buffer - self.target_tick_buffer as i64;

        // Speedup/slowdown time to get to target tick buffer.
        // For every tick above/below target tick we speedup/slowdown time by 2% (additif).
        self.time_dilation = (delta_target_tick_buffer as f32).mul_add(0.02, 1.0);
        self.delta += (delta / UPDATE_INTERVAL.as_secs_f32()) * self.time_dilation;

        // Increment tick.
        while self.delta >= 1.0 {
            self.tick += 1;
            self.delta -= 1.0;
        }

        let current_tick = self.tick;

        // Remove obselete fleet states.
        for fleets_forget in self
            .fleets_forget_buffer
            .drain_filter(|fleets_forget| fleets_forget.tick <= current_tick)
        {
            for small_id in fleets_forget.to_forget.into_iter() {
                if self.fleets_state.remove(&small_id).is_none() || self.fleets_state.remove(&small_id).is_none() {
                    warn!("Got order to remove {}, but it is not added. Ignoring...", small_id);
                }
            }
        }

        // Consume fleets infos.
        for fleets_infos in self
            .fleets_infos_buffer
            .drain_filter(|fleets_infos| fleets_infos.tick <= current_tick)
        {
            for fleet_infos in fleets_infos.infos {
                let small_id = fleet_infos.small_id;
                self.fleets_state
                    .entry(small_id)
                    .or_insert(FleetState::new(current_tick))
                    .fleet_infos = fleet_infos;
            }
        }

        // Consume fleets positions.
        for fleets_position in self
            .fleets_position_buffer
            .drain_filter(|fleets_position| fleets_position.tick <= current_tick)
        {
            // Update the client state.
            if let Some(fleet_state) = self.fleets_state.get_mut(&0) {
                fleet_state.update(fleets_position.tick, fleets_position.client_position);
            }

            // Update each entities position.
            for (small_id, position) in fleets_position.relative_fleets_position {
                if let Some(fleet_state) = self.fleets_state.get_mut(&small_id) {
                    // Convert from compressed relative position to world position.
                    let position = position.to_vec2(common::METASCAPE_RANGE) + fleets_position.client_position;
                    fleet_state.update(fleets_position.tick, position);
                } else {
                    debug!("missing fleet state.");
                }
            }
        }

        // Send client packets to server.
        self.send_timer -= delta;
        if self.send_timer <= 0.0 {
            self.send_timer = 0.010;

            // Send wish position.
            if player_inputs.primary {
                self.connection_manager.send(&ClientPacket::MetascapeWishPos {
                    wish_pos: player_inputs.global_mouse_position,
                    movement_multiplier: 1.0,
                });
            }

            // Flush packets.
            quit |= !self.connection_manager.flush()
        }

        quit
    }

    pub fn render(&mut self, owner: &Node2D) {
        let timef = self.tick as f32 + self.delta;

        // Get the position of our fleet.
        let pos = self
            .fleets_state
            .get(&0)
            .map(|fleet_state| fleet_state.current_position)
            .unwrap_or_default();

        // Debug draw fleets.
        for (small_id, fleet_state) in self.fleets_state.iter_mut() {
            let fade = ((self.tick as f32 - fleet_state.discovered_tick as f32) * 0.1).min(1.0);

            // Interpolate position.
            let pos = fleet_state.get_interpolated_pos(timef);

            let r = (*small_id % 7) as f32 / 7.0;
            let g = (*small_id % 11) as f32 / 11.0;
            let b = (*small_id % 13) as f32 / 13.0;
            let a = fade * 0.5;

            // Draw fleet radius.
            owner.draw_arc(
                pos.to_godot_scaled(),
                (0.25 * GAME_TO_GODOT_RATIO).into(),
                0.0,
                std::f64::consts::TAU,
                16,
                Color { r, g, b, a },
                1.0,
                false,
            );

            // Draw each ships.
            for (i, ship_base_id) in fleet_state.fleet_infos.composition.iter().enumerate() {
                owner.draw_circle(
                    (pos + Vec2::new(i as f32 * 0.1, 0.0)).to_godot_scaled(),
                    (0.1 * GAME_TO_GODOT_RATIO).into(),
                    Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a,
                    },
                );
            }
        }

        // Debug draw systems.
        let screen_collider = Collider::new(pos, self.configs.system_draw_distance, ());
        self.systems_acceleration.intersect_collider(screen_collider, |other| {
            let system = self.systems.systems.get(&other.id).unwrap();

            // Draw system bound.
            owner.draw_arc(
                system.position.to_godot_scaled(),
                (system.radius * GAME_TO_GODOT_RATIO).into(),
                0.0,
                std::f64::consts::TAU,
                32,
                Color {
                    r: 0.95,
                    g: 0.95,
                    b: 1.0,
                    a: 0.5,
                },
                4.0,
                false,
            );

            // Draw star.
            let (r, g, b) = match system.star.star_type {
                common::system::StarType::Star => (1.0, 0.2, 0.0),
                common::system::StarType::BlackHole => (0.0, 0.0, 0.0),
                common::system::StarType::Nebula => (0.0, 0.0, 0.0),
            };
            owner.draw_circle(
                system.position.to_godot_scaled(),
                (system.star.radius * GAME_TO_GODOT_RATIO).into(),
                Color { r, g, b, a: 0.5 },
            );

            // Draw planets.
            for planet in system.planets.iter() {
                owner.draw_circle(
                    planet
                        .relative_orbit
                        .to_position(timef, system.position)
                        .to_godot_scaled(),
                    (planet.radius * GAME_TO_GODOT_RATIO).into(),
                    Color {
                        r: 0.0,
                        g: 0.5,
                        b: 1.0,
                        a: 0.5,
                    },
                );
            }

            false
        });
    }
}
