pub mod connection_wrapper;
pub mod states_manager;

pub use connection_wrapper::*;

use self::states_manager::StatesManager;
use crate::configs::Configs;
use crate::constants::*;
use crate::input_handler::PlayerInputs;
use crate::time_manager::*;
use crate::util::*;
use ::utils::acc::*;
use common::idx::*;
use common::net::*;
use common::system::*;
use gdnative::api::*;
use gdnative::prelude::*;
use glam::Vec2;

pub enum MetascapeSignal {
    /// Disconnected from the server.
    Disconnected {
        reason: DisconnectedReason,
    },
    HasFleetChanged(bool),
}

pub struct ClientMetascape {
    systems: Systems,
    systems_acceleration: AccelerationStructure<Circle, SystemId>,

    pub connection: ConnectionClientSideWrapper,
    send_timer: f32,
    last_metascape_state_ack: u32,
    last_inputs: ClientInputsType,

    pub time_manager: TimeManager,
    pub states_manager: StatesManager,

    configs: Configs,
}
impl ClientMetascape {
    pub fn new(
        connection: ConnectionClientSideWrapper,
        client_id: ClientId,
        configs: Configs,
        systems: Systems,
    ) -> Self {
        Self {
            systems_acceleration: systems.create_acceleration_structure(),
            systems,
            connection,
            send_timer: 0.0,
            last_metascape_state_ack: 0,
            last_inputs: ClientInputsType::default(),
            time_manager: TimeManager::new(configs.time_manager_configs.to_owned()),
            states_manager: StatesManager::new(client_id.to_fleet_id()),
            configs,
        }
    }

    /// Return the signals triggered from this update.
    pub fn update(&mut self, delta: f32, player_inputs: &PlayerInputs) -> Vec<MetascapeSignal> {
        let mut signals = Vec::new();
        // Used to avoid filling signals with disconnect signals.
        let mut disconnect = false;

        // Handle server packets.
        self.connection.recv_packets(|packet| match packet {
            ServerPacket::Invalid => {
                if !disconnect {
                    signals.push(MetascapeSignal::Disconnected {
                        reason: DisconnectedReason::DeserializeError,
                    });
                    disconnect = true;
                }
            }
            ServerPacket::DisconnectedReason(reason) => {
                if !disconnect {
                    signals.push(MetascapeSignal::Disconnected { reason });
                    disconnect = true;
                }
            }
            ServerPacket::FleetsInfos(fleets_infos) => {
                log::debug!("{:#?}", &fleets_infos);
                self.states_manager.fleets_infos_buffer.push(fleets_infos);
            }
            ServerPacket::MetascapeState(metascape_state) => {
                self.time_manager.maybe_max_tick(metascape_state.tick);
                self.last_metascape_state_ack = self.last_metascape_state_ack.max(metascape_state.tick);
                self.states_manager.metascape_state_buffer.push(metascape_state);
            }
            ServerPacket::ConnectionQueueLen(_) => {}
            ServerPacket::LoginResponse(_) => {}
        });

        self.states_manager.update(self.time_manager.tick);

        // Handle client's inputs.
        if player_inputs.primary {
            self.last_inputs = ClientInputsType::Metascape {
                wish_pos: player_inputs.global_mouse_position,
                movement_multiplier: 1.0,
            };
        }

        // Send client packets to server.
        self.send_timer -= delta;
        if self.send_timer < 0.0 {
            self.send_timer = common::TICK_DURATION.as_secs_f32();

            // Send inputs
            self.connection.send_unreliable(&ClientPacket::ClientInputs {
                last_metascape_state_ack: self.last_metascape_state_ack,
                inputs: self.last_inputs,
            });

            // Clear client inputs.
            self.last_inputs = ClientInputsType::default();

            // Flush packets.
            if self.connection.flush() && !disconnect {
                signals.push(MetascapeSignal::Disconnected {
                    reason: DisconnectedReason::LostConnection,
                });
                disconnect = true;
            }
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
            let pos = fleet_state.get_interpolated_pos(time_manager);

            let r = ((fleet_id.0 + 5) % 7) as f32 / 7.0;
            let g = ((fleet_id.0 + 4) % 11) as f32 / 11.0;
            let b = ((fleet_id.0 + 3) % 5) as f32 / 5.0;
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
