pub mod render;
mod runner;

use self::render::{BattlescapeRender, RenderBattlescapeEventHandler};
use self::runner::RunnerHandle;
use super::*;
use crate::battlescape::events::BattlescapeEventHandlerTrait;
use crate::battlescape::*;
use crate::client_config::ClientConfig;
use crate::metascape::BattlescapeId;
use crate::player_inputs::PlayerInputs;
use crate::time_manager::*;
use crate::util::*;
use godot::engine::{packed_scene, CanvasLayer};
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
    client_id: ClientId,
    runner_handle: RunnerHandle,
    render: BattlescapeRender,
    fleets: Fleets,
    ship_selection: ShipSelection,
    client_type: ClientType,
    inputs: PlayerInputs,
    wish_cmds: Commands,
    last_cmds_send: f32,
    /// Flag telling if we are very far behind.
    /// This will disable rendering and inputs to speed up simulation.
    catching_up: bool,
    time_manager: TimeManager<{ DT_MS }>,
    cmd_tick: u64,
    events: Vec<ClientBattlescapeEventHandler>,
    replay: Replay,
    #[base]
    base: Base<Node2D>,
}
impl ClientBattlescape {
    pub fn new(
        replay: Replay,
        client_config: &ClientConfig,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Gd<Self> {
        // TODO: Take latest jump point.
        let bc = Battlescape::new(replay.initial_state.clone());

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
                runner_handle: RunnerHandle::new(bc),
                replay,
                ship_selection: ShipSelection::new(&mut base),
                wish_cmds: Default::default(),
                time_manager: TimeManager::new(config),
                client_type,
                inputs: Default::default(),
                last_cmds_send: 0.0,
                base,
                events: Default::default(),
                catching_up: true,
                cmd_tick: 0,
                fleets: Default::default(),
            }
        })
    }

    pub fn battlescape_id(&self) -> BattlescapeId {
        self.replay.battlescape_id
    }
}
#[godot_api]
impl ClientBattlescape {
    #[func]
    fn can_cheat(&self) -> bool {
        self.client_type == ClientType::LocalCheat
    }

    // #[func]
    // fn sv_add_ship(&mut self, fleet_idx: u32, ship_idx: u32) {
    //     if self.can_cheat() {
    //         self.wish_cmds.push(SvAddShip {
    //             fleet_id: FleetId(fleet_idx as u64),
    //             ship_idx,
    //             prefered_spawn_point: fleet_idx,
    //         });
    //     } else {
    //         log::warn!("Tried to cheat.");
    //     }
    // }

    #[func]
    fn get_ship_selection(&mut self) -> Gd<CanvasLayer> {
        self.ship_selection.node.share()
    }

    #[func]
    fn fleet_ship_selected(&mut self, selected: PackedInt64Array) {
        for mut selection_idx in selected.to_vec().into_iter().map(|i| i as usize) {
            let mut fleet_idx = 0;
            loop {
                if selection_idx < self.fleets[fleet_idx].ships.len() {
                    break;
                }
                selection_idx -= self.fleets[fleet_idx].ships.len();
                fleet_idx += 1;
            }

            let add_ship = SvAddShip {
                fleet_id: *self.fleets.get_index(fleet_idx).unwrap().0,
                ship_idx: selection_idx as u32,
                prefered_spawn_point: 0,
            };

            if self.can_cheat() {
                self.wish_cmds.push(add_ship);
            } else {
                self.wish_cmds.push(AddShip {
                    caller: self.client_id,
                    add_ship,
                });
            }
        }

        self.ship_selection.hide();
    }

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

        while let Some(cmds) = self.replay.get_cmds(self.cmd_tick + 1) {
            if self.events.len() > 10 {
                break;
            }
            self.runner_handle.step(
                cmds.to_owned(),
                ClientBattlescapeEventHandler::new(was_catching_up),
            );
            self.cmd_tick += 1;
        }

        // Handle events.
        for event in self.events.iter_mut() {
            if !event.handled {
                event.handled = true;

                for (fleet_id, new_fleet) in event.new_fleet.drain() {
                    // Add fleet to ship selection.
                    for ship in new_fleet.ships.iter() {
                        self.ship_selection.add_ship(ship);
                    }
                    
                    self.fleets.insert(fleet_id, new_fleet);
                }

                for (fleet_id, ship_idx, state) in event.ship_state_changes.drain(..) {
                    self.fleets.get_mut(&fleet_id).unwrap().ships[ship_idx].state = state;

                    // Get the ship idx in the ship selection.
                    let mut idx = ship_idx;
                    for (other_fleet_id, other_fleet) in self.fleets.iter() {
                        if fleet_id == *other_fleet_id {
                            break;
                        }
                        idx += other_fleet.ships.len();
                    }

                    self.ship_selection.update_ship_state(idx as i64, state);
                }
            }

            if event.tick <= self.time_manager.tick && !event.render_handled {
                event.render_handled = true;

                self.render.handle_event(event);
            }
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
                    caller: self.render.client_id,
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

struct ShipSelection {
    node: Gd<CanvasLayer>,
}
impl ShipSelection {
    const SHIP_SELECTION_PATH: &str = "res://ui/ship_selection.tscn";

    fn new(bs: &mut Gd<Node2D>) -> Self {
        let mut node = load::<PackedScene>(Self::SHIP_SELECTION_PATH)
            .instantiate(packed_scene::GenEditState::GEN_EDIT_STATE_DISABLED)
            .unwrap();
        add_child_node_node(&mut bs.share().upcast(), node.share());
        node.set("bs".into(), bs.to_variant());

        Self { node: node.cast() }
    }

    fn hide(&mut self) {
        self.node.hide();
    }

    fn show(&mut self) {
        self.node.show();
    }

    fn add_ship(&mut self, ship: &bc_fleet::BattlescapeFleetShip) {
        let ship_data = ship.original_ship.ship_data_id.data();
        let icon = ship_data.render_node.get_texture().unwrap().to_variant();
        let size_factor = 1.0f64.to_variant(); // TODO: size factor
        let tooptip = ship_data.display_name.to_variant();
        let cost = 10i64.to_variant(); // TODO: cost
        let destroyed = match ship.state {
            bc_fleet::FleetShipState::Ready => false,
            bc_fleet::FleetShipState::Spawned => true,
            bc_fleet::FleetShipState::Removed(_) => true,
            bc_fleet::FleetShipState::Destroyed => true,
        }
        .to_variant();

        // add_ship(icon: Texture2D, size_factor: float, tooptip: String, cost: int, destroyed: bool)
        self.node.call(
            "add_ship".into(),
            &[icon, size_factor, tooptip, cost, destroyed],
        );
    }

    fn update_ship_state(&mut self, idx: i64, state: bc_fleet::FleetShipState) {
        match state {
            bc_fleet::FleetShipState::Ready => {
                // TODO: Do we ever need to go back to ready?
                // ship_set_ready(idx: int)
                self.node.call("ship_set_ready".into(), &[idx.to_variant()]);
            }
            bc_fleet::FleetShipState::Spawned => {
                // ship_set_spawned(idx: int)
                self.node
                    .call("ship_set_spawned".into(), &[idx.to_variant()]);
            }
            bc_fleet::FleetShipState::Removed(_) => {
                // ship_set_removed(idx: int)
                self.node
                    .call("ship_set_removed".into(), &[idx.to_variant()]);
            }
            bc_fleet::FleetShipState::Destroyed => {
                // ship_set_destroyed(idx: int)
                self.node
                    .call("ship_set_destroyed".into(), &[idx.to_variant()]);
            }
        }
    }

    fn set_max_active_cost(&mut self, value: i64) {
        // set_max_active_cost(value: int)
        self.node
            .call("set_max_active_cost".into(), &[value.to_variant()]);
    }
}
impl Drop for ShipSelection {
    fn drop(&mut self) {
        self.node.queue_free();
    }
}

#[derive(Default)]
pub struct ClientBattlescapeEventHandler {
    handled: bool,
    render_handled: bool,

    tick: u64,
    battle_over: bool,

    _new_fleet: Vec<FleetId>,
    new_fleet: AHashMap<FleetId, battlescape::bc_fleet::BattlescapeFleet>,
    ship_state_changes: Vec<(FleetId, usize, bc_fleet::FleetShipState)>,

    render: RenderBattlescapeEventHandler,
}
impl ClientBattlescapeEventHandler {
    fn new(take_full: bool) -> Self {
        Self {
            render: RenderBattlescapeEventHandler::new(take_full),
            ..Default::default()
        }
    }
}
impl BattlescapeEventHandlerTrait for ClientBattlescapeEventHandler {
    fn stepped(&mut self, bc: &Battlescape) {
        self.tick = bc.tick;

        if self.render.take_full {
            self.new_fleet = AHashMap::from_iter(bc.fleets.iter().map(|(k, v)| (*k, v.clone())));
        } else {
            for fleet_id in self._new_fleet.iter() {
                self.new_fleet
                    .insert(*fleet_id, bc.fleets.get(fleet_id).unwrap().clone());
            }
        }
        self._new_fleet.clear();

        self.render.stepped(bc);
    }

    fn fleet_added(&mut self, fleet_id: FleetId) {
        self._new_fleet.push(fleet_id);

        self.render.fleet_added(fleet_id);
    }

    fn ship_destroyed(&mut self, fleet_id: FleetId, ship_index: usize) {
        self.ship_state_changes
            .push((fleet_id, ship_index, bc_fleet::FleetShipState::Destroyed));

        self.render.ship_destroyed(fleet_id, ship_index);
    }

    fn entity_removed(
        &mut self,
        entity_id: battlescape::EntityId,
        entity: battlescape::entity::Entity,
    ) {
        self.render.entity_removed(entity_id, entity);
    }

    fn hull_removed(&mut self, entity_id: EntityId, hull_index: usize) {
        self.render.hull_removed(entity_id, hull_index);
    }

    fn entity_added(
        &mut self,
        entity_id: battlescape::EntityId,
        entity: &battlescape::entity::Entity,
    ) {
        self.render.entity_added(entity_id, entity);
    }

    fn battle_over(&mut self) {
        self.battle_over = true;

        self.render.battle_over();
    }
}
