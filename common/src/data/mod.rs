use self::ship::*;
use self::weapon::*;
use crate::fleet::*;
use crate::idx::*;

pub mod ship;
pub mod weapon;

static mut DATA: *mut Data = std::ptr::null_mut();

/// ## Panic:
/// Data should be initialised.
pub fn data() -> &'static Data {
    unsafe { DATA.as_ref().expect("DATA is not initialised") }
}

pub struct Data {
    pub starting_fleets: Vec<FleetComposition>,
    pub ships: Vec<ShipBase>,
    pub weapons: Vec<WeaponBase>,
}
impl Data {
    /// Store this `Data` in a global static variable.
    /// Accessed through `common::data()`
    pub fn init(self) {
        unsafe {
            // Drop the previous data.
            if !DATA.is_null() {
                drop(Box::from_raw(DATA));
            }
            DATA = Box::into_raw(Box::new(self));
        }
    }
}
impl Default for Data {
    fn default() -> Self {
        Self {
            ships: vec![
                ShipBase {
                    name: "Frig".to_string(),
                    class: ShipClass::Frigate,
                    auto_combat_strenght: 100.0,
                },
                ShipBase {
                    name: "Destro".to_string(),
                    class: ShipClass::Destroyer,
                    auto_combat_strenght: 300.0,
                },
            ],
            weapons: vec![
                WeaponBase {
                    name: "Pee shooter".to_string(),
                    size: WeaponSize::Light,
                },
                WeaponBase {
                    name: "Potato laucher".to_string(),
                    size: WeaponSize::Medium,
                },
            ],
            starting_fleets: vec![FleetComposition {
                ships: vec![ShipInfos {
                    ship_base: ShipBaseId::from_raw(0),
                    hp: 1.0,
                    state: 1.0,
                    weapon_bases: vec![WeaponBaseId::from_raw(0)],
                }],
            }],
        }
    }
}
