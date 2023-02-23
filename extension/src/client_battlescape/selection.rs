use super::*;
use godot::engine::packed_scene;
use godot::engine::CanvasLayer;

struct StaticGodotString(pub GodotString);
unsafe impl Send for StaticGodotString {}
unsafe impl Sync for StaticGodotString {}

#[derive(GodotClass)]
#[class(base=CanvasLayer)]
pub struct ShipSelection {
    fleet_ship_idx: AHashMap<FleetShip, i64>,
    idx_fleet_ship: AHashMap<i64, FleetShip>,
    selection: AHashSet<i64>,
    #[base]
    base: Base<CanvasLayer>,
}
impl ShipSelection {
    const SHIP_SELECTION_PATH: &str = "res://ui/ship_selection.tscn";

    pub fn new_child(bs: &Gd<Node2D>) -> Gd<Self> {
        let node = load::<PackedScene>(Self::SHIP_SELECTION_PATH)
            .instantiate(packed_scene::GenEditState::GEN_EDIT_STATE_DISABLED)
            .unwrap()
            .cast::<Self>();
        add_child(bs, &node);
        node
    }

    pub fn add_ship_internal(
        &mut self,
        fleet_ship: FleetShip,
        ship: &bs_fleet::BattlescapeFleetShip,
    ) {
        let ship_data = ship.original_ship.ship_data_id.data();
        let icon = ship_data.texture.share();
        let size_factor = 1.0f64; // TODO: size factor
        let tooptip = GodotString::from(ship_data.display_name.as_str()); // TODO: Custom name
        let cost = 0i64; // TODO: cost

        let idx = self.add_ship(icon, size_factor, tooptip, cost);
        self.fleet_ship_idx.insert(fleet_ship, idx);
        self.idx_fleet_ship.insert(idx, fleet_ship);

        self.update_ship_state(fleet_ship, ship.state);
    }

    pub fn update_ship_state(&mut self, fleet_ship: FleetShip, state: bs_fleet::FleetShipState) {
        let idx = *self.fleet_ship_idx.get(&fleet_ship).unwrap();

        match state {
            bs_fleet::FleetShipState::Ready => self.ship_set_ready(idx),
            bs_fleet::FleetShipState::Spawned => self.ship_set_spawned(idx),
            bs_fleet::FleetShipState::Removed => self.ship_set_removed(idx),
            bs_fleet::FleetShipState::Destroyed => self.ship_set_destroyed(idx),
        }
    }

    fn add_ship(
        &mut self,
        icon: Gd<godot::engine::Texture2D>,
        size_factor: f64,
        tooptip: GodotString,
        cost: i64,
    ) -> i64 {
        self.call(
            "add_ship".into(),
            &[
                icon.to_variant(),
                size_factor.to_variant(),
                tooptip.to_variant(),
                cost.to_variant(),
            ],
        )
        .to()
    }

    fn ship_set_ready(&mut self, idx: i64) {
        self.call("ship_set_ready".into(), &[idx.to_variant()]);
    }

    fn ship_set_spawned(&mut self, idx: i64) {
        self.call("ship_set_spawned".into(), &[idx.to_variant()]);
    }

    fn ship_set_removed(&mut self, idx: i64) {
        self.call("ship_set_removed".into(), &[idx.to_variant()]);
    }

    fn ship_set_destroyed(&mut self, idx: i64) {
        self.call("ship_set_destroyed".into(), &[idx.to_variant()]);
    }

    pub fn set_max_active_cost(&mut self, value: i64) {
        self.call("set_max_active_cost".into(), &[value.to_variant()]);
    }
}
#[godot_api]
impl ShipSelection {
    #[func]
    pub fn get_parent_battlescape(&mut self) -> Gd<ClientBattlescape> {
        self.get_parent().unwrap().cast()
    }

    #[func]
    fn select(&mut self, idx: i64) {
        self.selection.insert(idx);
    }

    #[func]
    fn deselect(&mut self, idx: i64) {
        self.selection.remove(&idx);
    }

    #[func]
    fn clear_selection(&mut self) {
        self.selection.clear();
    }

    #[func]
    fn spawn_selected(&mut self) {
        self.hide();

        let mut _bs = self.get_parent_battlescape();
        let mut bs = _bs.bind_mut();

        for selection_idx in self.selection.drain() {
            let fleet_ship = self.idx_fleet_ship.get(&selection_idx).unwrap();

            let add_ship = SvAddShip {
                fleet_id: fleet_ship.0,
                ship_idx: fleet_ship.1 as u32,
            };

            if bs.can_cheat() {
                bs.add_cmd(add_ship);
            } else {
                let caller = bs.client_id;
                bs.add_cmd(AddShip { caller, add_ship });
            }
        }
    }
}
#[godot_api]
impl GodotExt for ShipSelection {
    fn init(base: Base<CanvasLayer>) -> Self {
        Self {
            fleet_ship_idx: Default::default(),
            idx_fleet_ship: Default::default(),
            selection: Default::default(),
            base,
        }
    }
}
