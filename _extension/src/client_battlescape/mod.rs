pub mod draw_shapes;
mod render;
mod runner;
mod selection;

use self::draw_shapes::*;
use self::render::{BattlescapeRender, RenderBattlescapeEventHandler};
use self::runner::RunnerHandle;
use self::selection::ShipSelection;
use super::*;
use crate::battlescape::events::BattlescapeEventHandlerTrait;
use crate::battlescape::*;
use crate::client_config::ClientConfig;
use crate::metascape::BattlescapeId;
use crate::player_inputs::PlayerInputs;
use crate::time_manager::*;
use crate::util::*;
use godot::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientType {
    /// Wish cmds are applied directly.
    Local,
    /// Local with the right to cheat.
    LocalCheat,
    /// Already has all the cmds.
    Replay,
    /// Send wish cmds to the server. Receive cmds from the server.
    Client,
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct ClientBattlescape {
    pub client_id: ClientId,
    runner_handle: RunnerHandle,
    render: BattlescapeRender,
    fleets: Fleets,
    ship_selection: Gd<ShipSelection>,
    client_type: ClientType,

    inputs: PlayerInputs,
    wish_cmds: Commands,
    last_cmds_send: f32,

    /// Flag telling if we are very far behind.
    /// This will disable rendering and inputs to speed up simulation.
    catching_up: bool,
    time_manager: TimeManager<{ DT_MS }>,
    /// The last tick send to the runner thread.
    cmd_tick: u64,
    /// The last tick we received from the runner thread.
    tick: u64,
    events: Vec<ClientBattlescapeEventHandler>,
    replay: Replay,

    hash_on_tick: u64,
    dbg_draw_colliders: bool,
    draw_colliders: Gd<DrawColliders>,

    #[base]
    base: Base<Node2D>,
}
impl ClientBattlescape {
    // TODO: Take latest jump point.
    pub fn new(
        replay: Replay,
        client_config: &ClientConfig,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Gd<Self> {
        let config = match client_type {
            ClientType::Local => TimeManagerConfig::local(),
            ClientType::LocalCheat => TimeManagerConfig::local(),
            ClientType::Replay => TimeManagerConfig::very_smooth(),
            ClientType::Client => client_config.battlescape_time_manager_config,
        };

        Gd::with_base(|mut base: Base<Node2D>| {
            base.hide();

            Self {
                client_id,
                render: BattlescapeRender::new(client_id, base.share()),
                runner_handle: RunnerHandle::new(replay.initial_state.clone()),
                replay,
                ship_selection: ShipSelection::new_child(&base),
                wish_cmds: Default::default(),
                time_manager: TimeManager::new(config),
                client_type,
                inputs: Default::default(),
                last_cmds_send: 0.0,
                events: Default::default(),
                catching_up: true,
                cmd_tick: 0,
                tick: 0,
                fleets: Default::default(),
                hash_on_tick: 0,
                dbg_draw_colliders: false,
                draw_colliders: DrawColliders::new_child(&base),
                base,
            }
        })
    }

    pub fn battlescape_id(&self) -> BattlescapeId {
        self.replay.battlescape_id
    }

    pub fn add_cmd(&mut self, cmd: impl Command) {
        self.wish_cmds.push(cmd);
    }
}
#[godot_api]
impl ClientBattlescape {
    #[signal]
    fn hash_received();

    #[func]
    fn can_cheat(&self) -> bool {
        self.client_type == ClientType::LocalCheat
    }

    #[func]
    fn tick(&self) -> i64 {
        self.tick as i64
    }

    #[func]
    fn cmd_tick(&self) -> i64 {
        self.cmd_tick as i64
    }

    #[func]
    fn hash_on_tick(&mut self, on_tick: i64) {
        self.hash_on_tick = on_tick as u64;
    }

    /// ---------- Entity ----------

    // #[func]
    // fn get_entity_at(&mut self, position: Vector2) -> i64 {
    //     let mut selected_distance_squared = f32::MAX;
    //     let mut selected_id = -1;

    //     for (entity_id, entity) in self.render.entity_renders.iter() {
    //         let pos = entity.position.pos;
    //         let r = entity.entity_data_id.render_data().radius_aprox;
    //         let dist = pos.distance_squared_to(position);
    //         if r * r > dist && dist < selected_distance_squared {
    //             selected_distance_squared = dist;
    //             selected_id = entity_id.0 as i64;
    //         }
    //     }

    //     selected_id
    // }

    #[func]
    fn get_owned_entity_at(&mut self, position: Vector2) -> i64 {
        let mut selected_distance_squared = f32::MAX;
        let mut selected_id = -1;

        for (entity_id, entity) in self.render.entity_renders.iter() {
            if !entity.owner.is_some_and(|owner| owner == self.client_id) {
                continue;
            }

            let dist = entity.position().distance_squared_to(position);
            if entity.entity_data_id.render_data().radius_aprox.powi(2) > dist
                && dist < selected_distance_squared
            {
                selected_distance_squared = dist;
                selected_id = entity_id.0 as i64;
            }
        }

        selected_id
    }

    /// ---------- Commands ----------

    #[func]
    fn cmd_control_ship(&mut self, entity_id: i64) {
        self.add_cmd(SetClientControl {
            caller: self.client_id,
            entity_id: u32::try_from(entity_id).ok().map(|id| EntityId(id)),
        });
    }

    /// ---------- Godot ----------

    #[func]
    fn get_child_ship_selection(&mut self) -> Gd<ShipSelection> {
        self.ship_selection.share()
    }

    /// ---------- Debug ----------

    #[func]
    fn dbg_print_fleets(&self) {
        log::info!("Printing fleets");
        for (fleet_id, fleet) in self.fleets.iter() {
            godot_print!("Fleet {:?}:", fleet_id);
            godot_print!("  Owner: {:?}", fleet.owner);
            godot_print!("  Team: {:?}", fleet.team);
            for (ship_idx, ship) in fleet.ships.iter().enumerate() {
                godot_print!("    Ship #{}:", ship_idx);
                godot_print!("      DataId: {:?}", ship.original_ship.ship_data_id);
                godot_print!("      State: {:?}", ship.state);
            }
        }
    }

    #[func]
    fn dbg_draw_colliders(&mut self, draw: bool) {
        self.dbg_draw_colliders = draw;
    }
}
#[godot_api]
impl GodotExt for ClientBattlescape {
    fn process(&mut self, delta: f64) {
        let delta = delta as f32;

        while let Ok(event) = self.runner_handle.runner_receiver.try_recv() {
            self.time_manager.maybe_max_tick(event.tick);
            self.events.push(event);
        }

        self.time_manager.update(delta);

        // TODO: Dynamic catching up.
        let was_catching_up: bool;
        if self.catching_up {
            self.catching_up = false;
            was_catching_up = true;
        } else {
            was_catching_up = false;
        }

        while self.cmd_tick - self.tick < 10 {
            let next_cmd_tick = self.cmd_tick + 1;

            if let Some(cmds) = self.replay.get_cmds(next_cmd_tick) {
                self.runner_handle.step(
                    cmds.to_owned(),
                    ClientBattlescapeEventHandler::new(
                        was_catching_up,
                        next_cmd_tick == self.hash_on_tick,
                        self.client_id,
                        self.dbg_draw_colliders,
                    ),
                );
                self.cmd_tick = next_cmd_tick;
            } else {
                break;
            }
        }

        // Handle events.
        let mut hashs = Vec::new();
        let mut draw_colliders: Option<Vec<DrawCollider>> = None;
        let mut ship_selection = self.ship_selection.bind_mut();
        for event in self.events.iter_mut() {
            if !event.handled {
                event.handled = true;

                self.tick = self.tick.max(event.tick);

                for (fleet_id, new_fleet) in event.new_fleet.drain() {
                    // Add fleet to ship selection.
                    for (ship_idx, ship) in new_fleet.ships.iter().enumerate() {
                        ship_selection.add_ship_internal((fleet_id, ship_idx), ship);
                    }

                    self.fleets.insert(fleet_id, new_fleet);
                }

                for (fleet_id, ship_idx, state) in event.ship_state_changes.drain(..) {
                    self.fleets.get_mut(&fleet_id).unwrap().ships[ship_idx].state = state;

                    ship_selection.update_ship_state((fleet_id, ship_idx), state);
                }

                if let Some(hash) = event.take_hash {
                    hashs.push(hash);
                }

                draw_colliders = event.take_colliders.take();
            }

            if event.tick <= self.time_manager.tick && !event.render_handled {
                event.render_handled = true;

                self.render.handle_event(event);
            }
        }
        drop(ship_selection);
        for hash in hashs {
            self.emit_signal("hash_received".into(), &[hash.to_variant()]);
        }
        if let Some(draw_colliders) = draw_colliders {
            self.draw_colliders
                .bind_mut()
                .enable_drawing(draw_colliders);
        } else if !self.dbg_draw_colliders {
            self.draw_colliders.bind_mut().disable_drawing();
        }

        // Remove previous events.
        self.events
            .retain(|event| event.tick + 1 >= self.time_manager.tick);

        let hidden = !self.base.is_visible();

        if hidden || self.catching_up {
            self.wish_cmds.clear();
            self.last_cmds_send = 0.0;
        } else {
            if self.events.len() >= 2 {
                let from = &self.events[0];
                let to = &self.events[1];
                self.render
                    .draw_lerp(from, to, self.time_manager.interpolation_weight());
            }
        }

        self.last_cmds_send += delta;
        if self.last_cmds_send > DT {
            self.last_cmds_send = 0.0;

            let mut cmds = std::mem::take(&mut self.wish_cmds);

            // Add inputs cmd
            if !hidden && self.client_type != ClientType::Replay {
                cmds.push(SetClientInput {
                    caller: self.client_id,
                    inputs: self.inputs.to_client_inputs(&self.base),
                });
            }

            if self.client_type == ClientType::Local || self.client_type == ClientType::LocalCheat {
                let tick = self.replay.next_needed_tick();
                self.replay.add_tick(tick, cmds);
            } else if !cmds.is_empty() && self.client_type != ClientType::Replay {
                // TODO: Send cmds to server
            }
        }
    }
}

#[derive(Default)]
pub struct ClientBattlescapeEventHandler {
    handled: bool,
    render_handled: bool,

    /// When Some, take a hash of the bs.
    take_hash: Option<u32>,

    take_colliders: Option<Vec<DrawCollider>>,

    tick: u64,
    battle_over: bool,

    _new_fleet: Vec<FleetId>,
    new_fleet: AHashMap<FleetId, battlescape::bs_fleet::BattlescapeFleet>,
    ship_state_changes: Vec<(FleetId, usize, bs_fleet::FleetShipState)>,

    render: RenderBattlescapeEventHandler,
}
impl ClientBattlescapeEventHandler {
    fn new(take_full: bool, take_hash: bool, client_id: ClientId, take_colliders: bool) -> Self {
        Self {
            render: RenderBattlescapeEventHandler::new(take_full, client_id),
            take_hash: take_hash.then_some(0),
            take_colliders: take_colliders.then_some(Vec::new()),
            ..Default::default()
        }
    }
}
impl BattlescapeEventHandlerTrait for ClientBattlescapeEventHandler {
    fn stepped(&mut self, bs: &Battlescape) {
        self.tick = bs.tick;

        if self.render.take_full {
            self.new_fleet = AHashMap::from_iter(bs.fleets.iter().map(|(k, v)| (*k, v.clone())));
        } else {
            for fleet_id in self._new_fleet.iter() {
                self.new_fleet
                    .insert(*fleet_id, bs.fleets.get(fleet_id).unwrap().clone());
            }
        }
        self._new_fleet.clear();

        if let Some(colliders) = &mut self.take_colliders {
            for (_, collider) in bs.physics.colliders.iter_enabled() {
                let collider_type = if let Some(ball) = collider.shape().as_ball() {
                    DrawColliderType::Circle {
                        radius: ball.radius * GODOT_SCALE,
                    }
                } else if let Some(cuboid) = collider.shape().as_cuboid() {
                    DrawColliderType::Cuboid {
                        half_size: cuboid.half_extents.to_godot_scaled(),
                    }
                } else if let Some(compound) = collider.shape().as_compound() {
                    let polygons = compound
                        .shapes()
                        .iter()
                        .map(|(pos, shape)| {
                            shape
                                .as_convex_polygon()
                                .unwrap()
                                .points()
                                .iter()
                                .map(|point| pos.transform_point(point).coords.to_godot_scaled())
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    DrawColliderType::CompoundPolygon { polygons }
                } else if let Some(polygon) = collider.shape().as_convex_polygon() {
                    DrawColliderType::Polygon {
                        points: polygon
                            .points()
                            .iter()
                            .map(|point| point.coords.to_godot_scaled())
                            .collect(),
                    }
                } else {
                    continue;
                };

                let pos = collider.position();

                let b = 1.0;

                colliders.push(DrawCollider {
                    collider_type,
                    color: Color {
                        r: 0.2,
                        g: 0.2,
                        b,
                        a: 0.5,
                    },
                    position: pos.translation.to_godot_scaled(),
                    rotation: pos.rotation.angle(), // TODO: Use hardward acceleration
                });
            }
        }

        self.render.stepped(bs);
    }

    fn fleet_added(&mut self, fleet_id: FleetId) {
        self._new_fleet.push(fleet_id);

        self.render.fleet_added(fleet_id);
    }

    fn ship_state_changed(
        &mut self,
        fleet_id: FleetId,
        ship_idx: usize,
        state: bs_fleet::FleetShipState,
    ) {
        self.ship_state_changes.push((fleet_id, ship_idx, state));

        self.render.ship_state_changed(fleet_id, ship_idx, state);
    }

    fn entity_removed(
        &mut self,
        entity_id: battlescape::EntityId,
        entity: battlescape::entity::Entity,
    ) {
        self.render.entity_removed(entity_id, entity);
    }

    fn entity_added(
        &mut self,
        entity_id: battlescape::EntityId,
        entity: &battlescape::entity::Entity,
        position: na::Isometry2<f32>,
    ) {
        self.render.entity_added(entity_id, entity, position);
    }

    fn battle_over(&mut self) {
        self.battle_over = true;

        self.render.battle_over();
    }
}

#[derive(Debug)]
pub struct EntityRenderData {
    /// Sprite2D
    pub render_scene: Gd<PackedScene>,
    pub child_sprite_idx: i64,
    /// In godot scale.
    pub radius_aprox: f32,
    // TODO: Engine placement.
}
impl Default for EntityRenderData {
    fn default() -> Self {
        Self {
            render_scene: load("res://fallback_entity_render.tscn"),
            child_sprite_idx: 0,
            radius_aprox: 0.5 * GODOT_SCALE,
        }
    }
}
