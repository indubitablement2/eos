use super::{bs_client::ClientInputs, *};
use crate::metascape::fleet::Fleet;

pub enum TypedCmd {
    AddFleet(AddFleet),
    SvAddShip(SvAddShip),
    AddShip(AddShip),
    SetClientInput(SetClientInput),
    SetClientControl(SetClientControl),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TypedCmds {
    AddFleet(Vec<AddFleet>),
    SvAddShip(Vec<SvAddShip>),
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
            TypedCmds::SvAddShip(v) => {
                if let TypedCmd::SvAddShip(c) = typed_cmd {
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
    fn apply(&self, bs: &mut Battlescape);
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
    fn apply(&self, bs: &mut Battlescape) {
        log::debug!(
            "Adding {:?} with {} ships owned by {:?}.",
            self.fleet_id,
            self.fleet.ships.len(),
            self.fleet.owner
        );
        bs.fleets.insert(
            self.fleet_id,
            BattlescapeFleet::from_fleet(self.fleet.to_owned(), self.team),
        );
        bs.events.fleet_added(self.fleet_id);
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::AddFleet(self)
    }

    fn server_only(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvAddShip {
    pub fleet_id: FleetId,
    pub ship_idx: u32,
}
impl Command for SvAddShip {
    fn apply(&self, bs: &mut Battlescape) {
        log::debug!("Adding ship #{} from {:?}", self.ship_idx, self.fleet_id);
        bs.add_fleet_ship(self.fleet_id, self.ship_idx as usize);
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::SvAddShip(self)
    }

    fn server_only(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddShip {
    pub caller: ClientId,
    pub add_ship: SvAddShip,
}
impl Command for AddShip {
    fn apply(&self, bs: &mut Battlescape) {
        // Check that caller own that fleet.
        if !bs
            .fleets
            .get(&self.add_ship.fleet_id)
            .and_then(|fleet| fleet.owner)
            .is_some_and(|fleet_owner| fleet_owner == self.caller)
        {
            return;
        }

        self.add_ship.apply(bs)
    }

    fn to_typed(self) -> TypedCmd {
        TypedCmd::AddShip(self)
    }

    fn server_only(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetClientInput {
    pub caller: ClientId,
    pub inputs: ClientInputs,
}
impl Command for SetClientInput {
    fn apply(&self, bs: &mut Battlescape) {
        let client = bs.clients.entry(self.caller).or_default();
        let mut inputs = self.inputs.clone();
        inputs.sanetize();
        client.client_inputs = inputs;
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
    pub caller: ClientId,
    pub entity_id: Option<EntityId>,
}
impl Command for SetClientControl {
    fn apply(&self, bs: &mut Battlescape) {
        let client = if let Some(client) = bs.clients.get_mut(&self.caller) {
            client
        } else {
            log::debug!(
                "Got SetClientControl cmd, but {:?} not found. Ignoring...",
                self.caller
            );
            return;
        };

        // Handle if we should simply remove control.
        let entity_id = if let Some(entity_id) = self.entity_id {
            entity_id
        } else {
            log::debug!("{:?} now control nothing", self.caller);
            client.control = None;
            return;
        };

        // Get the entity.
        let (entity_index, _, entity) = if let Some(entity) = bs.entities.get_full(&entity_id) {
            entity
        } else {
            log::debug!(
                "Got SetClientControl cmd, but {:?} not found. Ignoring...",
                entity_id
            );
            return;
        };

        // Check that caller own that entity.
        if let Some(owner) = entity.owner {
            if owner != self.caller {
                log::debug!(
                    "Got SetClientControl cmd, but {:?} is not owned by {:?}. Ignoring...",
                    entity_id,
                    self.caller
                );
                return;
            }
        } else {
            log::debug!(
                "Got SetClientControl cmd, but {:?} is not owned by anyone. Ignoring...",
                entity_id
            );
            return;
        }

        log::debug!("{:?} now control {:?}", self.caller, entity_id);
        client.control = Some(entity_id);
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
    pub fn apply(&self, bs: &mut Battlescape) {
        for cmds in self.0.iter() {
            match cmds {
                TypedCmds::AddFleet(c) => c.iter().for_each(|c| c.apply(bs)),
                TypedCmds::SvAddShip(c) => c.iter().for_each(|c| c.apply(bs)),
                TypedCmds::AddShip(c) => c.iter().for_each(|c| c.apply(bs)),
                TypedCmds::SetClientInput(c) => c.iter().for_each(|c| c.apply(bs)),
                TypedCmds::SetClientControl(c) => c.iter().for_each(|c| c.apply(bs)),
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
            TypedCmd::SvAddShip(c) => {
                self.0.push(TypedCmds::SvAddShip(vec![c]));
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
    pub battlescape_id: crate::metascape::BattlescapeId,
    pub initial_state: BattlescapeStateInit,
    cmds: Vec<Commands>,
    // TODO: sync points
}
impl Replay {
    pub fn new(
        battlescape_id: crate::metascape::BattlescapeId,
        initial_state: BattlescapeStateInit,
        cmds: Vec<Commands>,
    ) -> Self {
        Self {
            battlescape_id,
            initial_state,
            cmds,
        }
    }

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
