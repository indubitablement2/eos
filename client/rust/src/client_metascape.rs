use crate::configs::Configs;
use crate::connection_manager::ConnectionManager;
use crate::constants::*;
use crate::input_handler::PlayerInputs;
use crate::time_manager::*;
use crate::util::*;
use ::utils::acc::*;
use ahash::AHashMap;
use common::fleet::*;
use common::idx::*;
use common::net::packets::*;
use common::system::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

pub enum MetascapeSignal {
    /// Disconnected from the server.
    Disconnected {
        reason: DisconnectedReasonEnum,
    },
    HasFleetChanged {
        has_fleet: bool,
    },
}

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
                fleet_composition: FleetComposition::default(),
            },
        }
    }

    pub fn get_interpolated_pos(&self, tick: u32, tick_frac: f32, orbit_time: f32) -> Vec2 {
        if let Some(orbit) = self.fleet_infos.orbit {
            orbit.to_position(orbit_time)
        } else {
            let elapsed = (tick - self.previous_tick) as f32 + tick_frac;
            let range = (self.current_tick - self.previous_tick) as f32;
            let interpolation = elapsed / range;
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
    systems: Systems,
    systems_acceleration: AccelerationStructure<Circle, SystemId>,

    /// Send input to server. Receive command from server.
    pub connection_manager: ConnectionManager,
    send_timer: f32,

    pub time_manager: TimeManager,

    /// Client has `small_id` 0.
    fleets_state: AHashMap<u16, FleetState>,
    fleets_position_buffer: Vec<FleetsPosition>,
    fleets_infos_buffer: Vec<FleetsInfos>,
    fleets_forget_buffer: Vec<FleetsForget>,

    /// If we have a fleet.
    pub has_fleet: bool,

    configs: Configs,
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
            systems_acceleration: systems.create_acceleration_structure(),
            systems,
            connection_manager,
            send_timer: 0.0,
            time_manager: TimeManager::new(configs.time_manager_configs.to_owned()),

            fleets_state: Default::default(),
            fleets_position_buffer: Default::default(),
            fleets_infos_buffer: Default::default(),
            fleets_forget_buffer: Default::default(),
            has_fleet: false,
            configs,
        })
    }

    /// Return the signals triggered from this update.
    pub fn update(&mut self, delta: f32, player_inputs: &PlayerInputs) -> Vec<MetascapeSignal> {
        let mut signals = Vec::new();

        // Handle server packets.
        loop {
            match self.connection_manager.try_recv() {
                Ok(packet) => match ServerPacket::deserialize(&packet) {
                    ServerPacket::Invalid => {
                        signals.push(MetascapeSignal::Disconnected {
                            reason: DisconnectedReasonEnum::DeserializeError,
                        });
                        break;
                    }
                    ServerPacket::DisconnectedReason(reason) => {
                        signals.push(MetascapeSignal::Disconnected { reason });
                        break;
                    }
                    ServerPacket::ConnectionQueueLen(_) => todo!(),
                    ServerPacket::FleetsInfos(fleets_infos) => {
                        self.fleets_infos_buffer.push(fleets_infos);

                        // Maybe switch has_fleet.
                        if !self.has_fleet {
                            self.has_fleet = true;
                            signals.push(MetascapeSignal::HasFleetChanged { has_fleet: true });
                        }
                    }
                    ServerPacket::FleetsPosition(fleet_position) => {
                        self.time_manager.maybe_max_tick(fleet_position.tick);
                        self.fleets_position_buffer.push(fleet_position);
                    }
                    ServerPacket::FleetsForget(fleets_forget) => {
                        self.fleets_forget_buffer.push(fleets_forget);
                    }
                    ServerPacket::NoFleet => {
                        self.has_fleet = false;
                        signals.push(MetascapeSignal::HasFleetChanged { has_fleet: false });
                    }
                },
                Err(err) => {
                    if err.is_disconnected() {
                        signals.push(MetascapeSignal::Disconnected {
                            reason: DisconnectedReasonEnum::LostConnection,
                        });
                    }
                    break;
                }
            }
        }

        self.time_manager.update(delta);
        let current_tick = self.time_manager.tick;

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

                // TODO: Remove this. Just for testing.
                if self.fleets_state.contains_key(&small_id) {
                    log::error!("Already have {}", small_id);
                }

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
                    let position = position.to_vec2() + fleets_position.client_position;
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
            self.connection_manager.flush();
        }

        signals
    }

    pub fn render(&mut self, owner: &Node2D) {
        let orbit_time = self.time_manager.orbit_time();
        let tick = self.time_manager.tick;
        let tick_frac = self.time_manager.tick_frac;

        // Get the position of our fleet.
        let pos = self
            .fleets_state
            .get(&0)
            .map(|fleet_state| fleet_state.current_position)
            .unwrap_or_default();

        // Debug draw fleets.
        for (small_id, fleet_state) in self.fleets_state.iter_mut() {
            let fade = ((tick - fleet_state.discovered_tick)  as f32 * 0.1).min(1.0);

            // Interpolate position.
            let pos = fleet_state.get_interpolated_pos(tick, tick_frac, orbit_time);

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
            for (i, ship_infos) in fleet_state.fleet_infos.fleet_composition.ships.iter().enumerate() {
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
        let screen_collider = Circle::new(pos, self.configs.system_draw_distance);
        self.systems_acceleration.intersect(&screen_collider, |_, system_id| {
            let system = self.systems.systems.get(system_id).unwrap();

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
                        .to_position(orbit_time, system.position)
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
