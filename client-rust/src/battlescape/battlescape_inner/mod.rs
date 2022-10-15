pub mod physics;
pub mod state_init;

use gdnative::api::*;
use gdnative::prelude::*;
use physics::*;
use rand_xoshiro::Xoshiro256StarStar;
use rayon::prelude::*;
use state_init::*;

pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};

unsafe fn asd() {
    let o = gdnative::api::Object::new();
    let s = load::<Resource>("asd").unwrap();
    s.assume_safe().call("step", &[]);
}

pub struct BattlescapeInner {
    pub bound: f32,
    pub tick: u64,
    pub rng: rand_xoshiro::Xoshiro256StarStar,
    pub physics: Physics,
    pub scripts: Vec<Ref<Resource, Shared>>,
}
impl BattlescapeInner {
    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        Self {
            bound: battlescape_initial_state.bound,
            rng: Xoshiro256StarStar::seed_from_u64(battlescape_initial_state.seed),
            tick: 0,
            physics: Default::default(),
            scripts: Default::default(),
        }
    }

    pub fn step(&mut self) {
        // apply_commands::apply_commands(self, cmds);
        self.physics.step();
        self.scripts.par_iter().for_each(|s| unsafe {
            let b = s.assume_safe();
            if b.has_method("step") {
                b.call("step", &[]);
            }
        });
        let s = self.scripts.first().unwrap();
        // TODO: Handle events.
        self.tick += 1;
    }

    pub fn save(&self) -> Vec<u8> {
        todo!()
        // bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap_or_default()
    }

    pub fn load(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        todo!()
        // bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }
}
impl Default for BattlescapeInner {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
