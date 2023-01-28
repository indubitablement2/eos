use super::{bc_client::ClientInputs, *};
use crate::metascape::fleet::Fleet;

pub enum TypedCmd {
    AddFleet(AddFleet),
    AddShip(AddShip),
    SetClientInput(SetClientInput),
    SetClientControl(SetClientControl),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TypedCmds {
    AddFleet(Vec<AddFleet>),
    AddShip(Vec<AddShip>),
    SetClientInput(Vec<SetClientInput>),
    SetClientControl(Vec<SetClientControl>),
}
impl TypedCmds {
    pub fn try_push(&mut self, typed_cmd: TypedCmd) -> Option<TypedCmd> {
        match self {
            TypedCmds::AddFleet(v) => {
                if let TypedCmd::AddFleet(c) = typed_cmd {
                    v.push(c);
                    None
                } else {
                    Some(typed_cmd)
                }
            }
            TypedCmds::AddShip(v) => {
                if let TypedCmd::AddShip(c) = typed_cmd {
                    v.push(c);
                    None
                } else {
                    Some(typed_cmd)
                }
            }
            TypedCmds::SetClientInput(v) => {
                if let TypedCmd::SetClientInput(c) = typed_cmd {
                    v.push(c);
                    None
                } else {
                    Some(typed_cmd)
                }
            }
            TypedCmds::SetClientControl(v) => {
                if let TypedCmd::SetClientControl(c) = typed_cmd {
                    v.push(c);
                    None
                } else {
                    Some(typed_cmd)
                }
            }
        }
    }
}

pub trait Command {
    fn apply(&self, bc: &mut Battlescape);
    fn to_typed(self) -> TypedCmd;
    fn server_only(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFleet {
    pub fleet_id: FleetId,
    pub fleet: Fleet,
    pub team: u32,
}
impl Command for AddFleet {
    fn apply(&self, bc: &mut Battlescape) {
        bc.fleets.insert(
            self.fleet_id,
            BattlescapeFleet::from_fleet(self.fleet.to_owned(), self.team),
        );
        bc.events.fleet_added(self.fleet_id);
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::AddFleet(self)
    }

    fn server_only(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddShip {
    pub fleet_id: FleetId,
    pub ship_index: u32,
    pub prefered_spawn_point: u32,
}
impl Command for AddShip {
    fn apply(&self, bc: &mut Battlescape) {
        bc.add_fleet_ship(
            self.fleet_id,
            self.ship_index as usize,
            self.prefered_spawn_point as usize,
        );
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::AddShip(self)
    }

    fn server_only(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetClientInput {
    pub client_id: ClientId,
    pub inputs: ClientInputs,
}
impl Command for SetClientInput {
    fn apply(&self, bc: &mut Battlescape) {
        if let Some(client) = bc.clients.get_mut(&self.client_id) {
            let mut inputs = self.inputs.clone();
            inputs.sanetize();
            client.client_inputs = inputs;
        } else {
            log::warn!(
                "Got SetClientInput cmd, but {:?} not found. Ignoring...",
                self.client_id
            );
        }
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::SetClientInput(self)
    }

    fn server_only(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetClientControl {
    pub client_id: ClientId,
    pub entity_id: Option<EntityId>,
    // pub ship_id: Option<ShipId>,
}
impl Command for SetClientControl {
    fn apply(&self, bc: &mut Battlescape) {
        if let Some(client) = bc.clients.get_mut(&self.client_id) {
            let entity_id = if let Some(entity_id) = self.entity_id {
                entity_id
            } else {
                client.control = None;
                return;
            };

            if bc
                .entities
                .get(&entity_id)
                .and_then(|entity| entity.fleet_ship)
                .and_then(|(fleet_id, _)| bc.fleets.get(&fleet_id))
                .and_then(|fleet| fleet.owner)
                .is_some_and(|owner| owner == self.client_id)
            {
                client.control = Some(entity_id);
            } else {
                log::debug!(
                    "Got SetClientControl cmd, but {:?} does not own {:?}. Removing control...",
                    self.client_id,
                    entity_id
                );
                client.control = None;
            }
        } else {
            log::warn!(
                "Got SetClientControl cmd, but {:?} not found. Ignoring...",
                self.client_id
            );
        }
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::SetClientControl(self)
    }

    fn server_only(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Commands(Vec<TypedCmds>);
impl Commands {
    pub fn apply(&self, bc: &mut Battlescape) {
        for cmds in self.0.iter() {
            match cmds {
                TypedCmds::AddFleet(c) => c.iter().for_each(|c| c.apply(bc)),
                TypedCmds::AddShip(c) => c.iter().for_each(|c| c.apply(bc)),
                TypedCmds::SetClientInput(c) => c.iter().for_each(|c| c.apply(bc)),
                TypedCmds::SetClientControl(c) => c.iter().for_each(|c| c.apply(bc)),
            }
        }
    }

    pub fn push(&mut self, cmd: impl Command) {
        let mut typed_cmd = cmd.to_typed();

        for v in self.0.iter_mut() {
            if let Some(r) = v.try_push(typed_cmd) {
                typed_cmd = r
            } else {
                return;
            }
        }

        match typed_cmd {
            TypedCmd::AddFleet(c) => {
                self.0.push(TypedCmds::AddFleet(vec![c]));
            }
            TypedCmd::AddShip(c) => {
                self.0.push(TypedCmds::AddShip(vec![c]));
            }
            TypedCmd::SetClientInput(c) => {
                self.0.push(TypedCmds::SetClientInput(vec![c]));
            }
            TypedCmd::SetClientControl(c) => {
                self.0.push(TypedCmds::SetClientControl(vec![c]));
            }
        }
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Replay {
    pub initial_state: BattlescapeStateInit,
    cmds: Vec<Commands>,
    // TODO: sync points
}
impl Replay {
    pub fn get_cmds(&self, tick: u64) -> Option<&Commands> {
        if let Some(tick) = tick.checked_sub(1) {
            self.cmds.get(tick as usize)
        } else {
            // Tick 0 is imposible.
            log::warn!("Requested tick {} which is invalid. Ignoring...", tick);
            None
        }
    }

    /// Add the next needed tick.
    /// Return if the tick was added.
    pub fn add_tick(&mut self, tick: u64, cmds: Commands) -> bool {
        if tick == self.next_needed_tick() {
            self.cmds.push(cmds);
            true
        } else {
            false
        }
    }

    pub fn next_needed_tick(&self) -> u64 {
        self.cmds.len() as u64 + 1
    }

    /// May return tick 0 (empty replay) even though it does not exist.
    pub fn highest_available_tick(&self) -> u64 {
        self.cmds.len() as u64
    }
}
