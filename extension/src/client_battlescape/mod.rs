mod render;
mod runner;
mod selection;

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
                base,
                events: Default::default(),
                catching_up: true,
                cmd_tick: 0,
                tick: 0,
                fleets: Default::default(),
                hash_on_tick: 0,
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
    fn get_child_ship_selection(&mut self) -> Gd<ShipSelection> {
        self.ship_selection.share()
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

        while self.cmd_tick - self.tick < 10 {
            let next_cmd_tick = self.cmd_tick + 1;

            if let Some(cmds) = self.replay.get_cmds(next_cmd_tick) {
                self.runner_handle.step(
                    cmds.to_owned(),
                    ClientBattlescapeEventHandler::new(
                        was_catching_up,
                        next_cmd_tick == self.hash_on_tick
                    ),
                );
                self.cmd_tick = next_cmd_tick;
            } else {
                break;
            }
        }

        // Handle events.
        let mut hashs = Vec::new();
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

#[derive(Default)]
pub struct ClientBattlescapeEventHandler {
    handled: bool,
    render_handled: bool,

    /// When Some, take a hash of the bs.
    take_hash: Option<u32>,

    tick: u64,
    battle_over: bool,

    _new_fleet: Vec<FleetId>,
    new_fleet: AHashMap<FleetId, battlescape::bc_fleet::BattlescapeFleet>,
    ship_state_changes: Vec<(FleetId, usize, bc_fleet::FleetShipState)>,

    render: RenderBattlescapeEventHandler,
}
impl ClientBattlescapeEventHandler {
    fn new(take_full: bool, take_hash: bool) -> Self {
        Self {
            render: RenderBattlescapeEventHandler::new(take_full),
            take_hash: take_hash.then_some(0),
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
