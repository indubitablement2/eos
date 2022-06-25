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
use common::orbit::Orbit;
use common::system::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

pub enum MetascapeSignal {
    /// Disconnected from the server.
    Disconnected {
        reason: DisconnectedReasonEnum,
    },
    OwnedFleetsChanged {
        num: i32,
    },
    ControlChanged {
        index: i32,
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
    orbit_update_tick: u32,
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
            orbit_update_tick: discovered_tick,
            fleet_infos: FleetInfos {
                fleet_id: FleetId(0),
                small_id: 0,
                name: "".to_string(),
                orbit: None,
                fleet_composition: FleetComposition::default(),
            },
        }
    }

    pub fn get_interpolated_pos(&self, time_manager: &TimeManager, orbit_time: f32) -> Vec2 {
        if let Some(orbit) = self.fleet_infos.orbit {
            orbit.to_position(orbit_time)
        } else {
            let interpolation = time_manager.compute_interpolation(self.previous_tick, self.current_tick);
            self.previous_position.lerp(self.current_position, interpolation)
        }
    }

    fn update_position(&mut self, new_tick: u32, new_position: Vec2) {
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

        // Remove orbit.
        if new_tick >= self.orbit_update_tick {
            self.fleet_infos.orbit = None;
        }
    }

    fn update_fleet_infos(&mut self, fleet_infos_update_tick: u32, fleet_infos: FleetInfos) {
        self.fleet_infos = fleet_infos;
        self.orbit_update_tick = fleet_infos_update_tick;
    }

    fn update_fleet_composition(&mut self, fleet_composition: FleetComposition) {
        self.fleet_infos.fleet_composition = fleet_composition;
    }

    fn update_orbit(&mut self, update_tick: u32, orbit: Orbit) {
        if self.current_tick > update_tick {
            // Old/useless update.
            return;
        }
        self.fleet_infos.orbit = Some(orbit);
        self.orbit_update_tick = update_tick;
    }
}

pub struct Metascape {
    systems: Systems,
    systems_acceleration: AccelerationStructure<Circle, SystemId>,

    /// Send input to server. Receive command from server.
    pub connection_manager: ConnectionManager,
    send_timer: f32,

    pub time_manager: TimeManager,

    fleets_state: AHashMap<u16, FleetState>,
    fleets_position_buffer: Vec<FleetsPosition>,
    fleets_infos_buffer: Vec<FleetsInfos>,
    fleets_forget_buffer: Vec<FleetsForget>,

    /// The fleets we own, if any.
    pub owned_fleets: Vec<FleetId>,
    pub control: Option<FleetId>,

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
            owned_fleets: Default::default(),
            configs,
            control: None,
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
                        log::debug!("{:#?}", &fleets_infos);
                        self.fleets_infos_buffer.push(fleets_infos);
                    }
                    ServerPacket::FleetsPosition(fleet_position) => {
                        self.time_manager.maybe_max_tick(fleet_position.tick);
                        self.fleets_position_buffer.push(fleet_position);
                    }
                    ServerPacket::FleetsForget(fleets_forget) => {
                        log::debug!("Asked to forget {:?}", &fleets_forget);
                        self.fleets_forget_buffer.push(fleets_forget);
                    }
                    ServerPacket::OwnedFleets(owned_fleets) => {
                        self.owned_fleets = owned_fleets;
                        signals.push(MetascapeSignal::OwnedFleetsChanged {
                            num: self.owned_fleets.len() as i32,
                        });
                    }
                    ServerPacket::FleetControl(control) => {
                        self.control = control;
                        let index = if let Some(fleet_id) = self.control {
                            self.owned_fleet_index(fleet_id).map(|i| i as i32).unwrap_or(-1)
                        } else {
                            -1
                        };
                        signals.push(MetascapeSignal::ControlChanged { index })
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
                if self.fleets_state.remove(&small_id).is_none() {
                    warn!("Got order to remove {}, but it is not added. Ignoring...", small_id);
                }
            }
        }

        // Consume fleets infos.
        for fleets_infos in self
            .fleets_infos_buffer
            .drain_filter(|fleets_infos| fleets_infos.tick <= current_tick)
        {
            // Add new fleets.
            for fleet_infos in fleets_infos.new_fleets {
                let small_id = fleet_infos.small_id;

                self.fleets_state
                    .entry(small_id)
                    .or_insert(FleetState::new(current_tick))
                    .update_fleet_infos(fleets_infos.tick, fleet_infos);
            }

            // Update changed composition.
            for (small_id, fleet_composition) in fleets_infos.compositions_changed {
                if let Some(fleets_state) = self.fleets_state.get_mut(&small_id) {
                    fleets_state.update_fleet_composition(fleet_composition);
                } else {
                    log::error!("Received composition change for {}, but it is not found.", small_id);
                }
            }

            // Update changed orbit.
            for (small_id, orbit) in fleets_infos.orbits_changed {
                if let Some(fleets_state) = self.fleets_state.get_mut(&small_id) {
                    fleets_state.update_orbit(fleets_infos.tick, orbit);
                } else {
                    log::error!("Received orbit change for {}, but it is not found.", small_id);
                }
            }
        }

        // Consume fleets positions.
        for fleets_position in self
            .fleets_position_buffer
            .drain_filter(|fleets_position| fleets_position.tick <= current_tick)
        {
            // Update each entities position.
            for (small_id, position) in fleets_position.relative_fleets_position {
                if let Some(fleet_state) = self.fleets_state.get_mut(&small_id) {
                    // Convert from compressed relative position to world position.
                    let position = position.to_vec2() + fleets_position.origin;
                    fleet_state.update_position(fleets_position.tick, position);
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
        let time_manager = &self.time_manager;
        let tick = self.time_manager.tick;

        // Get the position of our fleet.
        let pos = self
            .fleets_state
            .get(&0)
            .map(|fleet_state| fleet_state.current_position)
            .unwrap_or_default();

        // Debug draw fleets.
        for (small_id, fleet_state) in self.fleets_state.iter_mut() {
            let fade = time_manager
                .compute_interpolation(fleet_state.discovered_tick, fleet_state.discovered_tick + 20)
                .min(1.0);

            // Interpolate position.
            let pos = fleet_state.get_interpolated_pos(time_manager, orbit_time);

            let r = (*small_id % 7) as f32 / 7.0;
            let g = (*small_id % 11) as f32 / 11.0;
            let b = fleet_state.fleet_infos.orbit.is_some() as i32 as f32;
            let a = fade * 0.75;

            // Draw fleet radius.
            owner.draw_arc(
                pos.to_godot_scaled(),
                (0.25 * GAME_TO_GODOT_RATIO).into(),
                0.0,
                std::f64::consts::TAU,
                16,
                Color { r, g, b, a },
                4.0,
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

    /// Return the index of a owned fleet.
    pub fn owned_fleet_index(&self, fleet_id: FleetId) -> Option<usize> {
        for (i, owned_fleet_id) in self.owned_fleets.iter().enumerate() {
            if owned_fleet_id.0 == fleet_id.0 {
                return Some(i);
            }
        }
        None
    }
}
