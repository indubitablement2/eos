use super::*;
use crate::metascape::fleet::Fleet;

pub trait Command {
    fn apply(&self, bc: &mut Battlescape);
    fn queue(self, cmds: &mut Commands);
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

    fn queue(self, cmds: &mut Commands) {
        cmds.add_fleet.push(self);
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

    fn queue(self, cmds: &mut Commands) {
        cmds.add_ship.push(self);
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SpawnShip {
//     pub player_id: u16,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SetClientControl {
//     pub client_id: ClientId,
//     pub ship_id: Option<ShipId>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SetClientInput {
//     pub client_id: ClientId,
//     pub inputs: bc_client::PlayerInput,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddFleet {
//     pub fleet_id: FleetId,
//     pub fleet: Fleet,
//     /// This should be an existing team.
//     /// If `None`, create a new team.
//     pub team: Option<Team>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ShipMoveOrder {
//     pub ship_id: Vec<ShipId>,
//     pub position: Vector2,
// }
// impl Command for ShipMoveOrder {
//     fn apply(&self, bc: &mut Battlescape, _events: &mut BcEvents) {
//         for ship_id in self.ship_id.iter() {
//             if let Some(ship) = bc.ships.get_mut(ship_id) {
//                 ship.wish_pos = Some(self.position);
//             }
//         }
//     }

//     fn queue(self, cmds: &mut Commands) {
//         cmds.ship_move_order.push(self)
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ShipJoin {
//     pub ship_save: ShipSave,
//     pub from_system: SystemId,
// }
// impl Command for ShipJoin {
//     fn apply(&self, bc: &mut Battlescape, events: &mut BcEvents) {
//         // TODO: Compute where the ship come from.
//         let translation = Vector2::zeros();
//         let rotation = 0.0;

//         self.ship_save.clone().to_ship(bc, translation, rotation);

//         events.ship_entered.push(self.ship_save.ship_id);
//     }

//     fn queue(self, cmds: &mut Commands) {
//         cmds.ship_join.push(self)
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct BuildStation {}
// impl Command for BuildStation {
//     fn apply(&self, bc: &mut Battlescape, events: &mut BcEvents) {
//         todo!()
//     }

//     fn queue(self, cmds: &mut Commands) {
//         cmds.build_station.push(self)
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Commands {
    pub add_fleet: Vec<AddFleet>,
    pub add_ship: Vec<AddShip>,
}
impl Commands {
    pub fn apply(&self, bc: &mut Battlescape) {
        for cmd in self.add_fleet.iter() {
            cmd.apply(bc);
        }
        for cmd in self.add_ship.iter() {
            cmd.apply(bc);
        }
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
