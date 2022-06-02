use self::ship::*;
use self::weapon::*;

pub mod ship;
pub mod weapon;

static mut DATA: *mut Data = std::ptr::null_mut();

/// Data should be initialized.
pub fn data() -> &'static Data {
    unsafe { &*DATA }
}

pub struct Data {
    pub ships: Vec<ShipBase>,
    pub weapons: Vec<WeaponBase>,
}
impl Data {
    pub fn init(self) {
        unsafe {
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
        }
    }
}
