use self::ship::*;
use self::weapon::*;

pub mod ship;
pub mod weapon;

static mut DATA: *mut Data = std::ptr::null_mut();

/// ## Panic: 
/// Data should be initialised.
pub fn data() -> &'static Data {
    unsafe { 
        DATA.as_ref().expect("DATA is not initialised")
    }
}

pub struct Data {
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
                Box::from_raw(DATA);
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
        }
    }
}
