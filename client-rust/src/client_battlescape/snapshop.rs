use battlescape::*;
use rapier2d::data::Arena;

#[derive(Default)]
pub struct BattlescapeSnapshot {
    pub tick: u64,
    pub bound: f32,
    pub hulls: Arena<Hull>,
    pub ships: Arena<Ship>,
    pub bodies: RigidBodySet,
}
impl BattlescapeSnapshot {
    pub fn take_snapshot(&mut self, bc: &Battlescape) {
        self.tick = bc.tick;
        self.bound = bc.bound;
        bc.hulls.clone_into(&mut self.hulls);
        bc.ships.clone_into(&mut self.ships);
        bc.physics.bodies.clone_into(&mut self.bodies);
    }
}