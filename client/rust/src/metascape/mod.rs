pub mod states_manager;

use crate::configs::Configs;
use crate::connection_manager::ConnectionManager;
use crate::constants::*;
use crate::input_handler::PlayerInputs;
use crate::time_manager::*;
use crate::util::*;
use ::utils::acc::*;
use common::idx::*;
use common::net::packets::*;
use common::system::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

use self::states_manager::StatesManager;

pub enum MetascapeSignal {
    /// Disconnected from the server.
    Disconnected {
        reason: DisconnectedReasonEnum,
    },
    HasFleetChanged(bool),
}

pub struct Metascape {
    systems: Systems,
    systems_acceleration: AccelerationStructure<Circle, SystemId>,

    /// Send input to server. Receive command from server.
    pub connection_manager: ConnectionManager,
    send_timer: f32,

    pub time_manager: TimeManager,
    pub states_manager: StatesManager,

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

        let client_fleet_id = connection_manager.client_id.to_fleet_id();

        Ok(Self {
            systems_acceleration: systems.create_acceleration_structure(),
            systems,
            connection_manager,
            send_timer: 0.0,
            time_manager: TimeManager::new(configs.time_manager_configs.to_owned()),
            states_manager: StatesManager::new(client_fleet_id),
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
                        log::debug!("{:#?}", &fleets_infos);
                        self.states_manager.fleets_infos_buffer.push(fleets_infos);
                    }
                    ServerPacket::FleetsPosition(fleet_position) => {
                        self.time_manager.maybe_max_tick(fleet_position.tick);
                        self.states_manager.fleets_position_buffer.push(fleet_position);
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

        self.states_manager.update(self.time_manager.tick);

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

        self.time_manager.update(delta);

        signals
    }

    pub fn render(&mut self, owner: &Node2D) {
        let orbit_time = self.time_manager.orbit_time();
        let time_manager = &self.time_manager;
        let states_manager = &self.states_manager;

        // Get the position of our fleet.
        let pos = states_manager
            .get_client_fleet()
            .map(|fleet_state| fleet_state.current_position)
            .unwrap_or_default();

        // Debug draw fleets.
        for (fleet_id, fleet_state) in states_manager.fleets_state.iter() {
            let fade = time_manager
                .compute_interpolation(fleet_state.discovered_tick, fleet_state.discovered_tick + 20)
                .min(1.0);

            // Interpolate position.
            let pos = fleet_state.get_interpolated_pos(time_manager, orbit_time);

            let r = (fleet_id.0 % 7) as f32 / 7.0;
            let g = (fleet_id.0 % 11) as f32 / 11.0;
            let b = fleet_state
                .fleet_infos
                .as_ref()
                .and_then(|fleet_infos| fleet_infos.orbit)
                .is_some() as i32 as f32;
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
            if let Some(fleet_infos) = &fleet_state.fleet_infos {
                for (i, ship_infos) in fleet_infos.fleet_composition.ships.iter().enumerate() {
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
            } else {
                // TODO: Draw dots...
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
