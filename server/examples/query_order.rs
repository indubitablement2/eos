use bevy_ecs::prelude::*;
use rand::random;

struct C1();
struct C2();
struct C3();

pub fn main() {
    let mut w = World::new();
    let mut schedule = Schedule::default();
    schedule.add_stage("1", SystemStage::parallel());
    schedule.add_system_to_stage("1", s.system());

    for _ in 0..1000 {
        let r = random::<u32>() % 3;
        let mut e = w.spawn();
        if r == 0 {
            e.insert(C1());
        } else if r == 1 {
            e.insert(C2());
        } else {
            e.insert(C3());
        }
    }

    schedule.run_once(&mut w);
}

fn s(query: Query<(Entity, &C1)>) {
    let mut last = Entity::new(0);
    query.for_each(|(e, _)| {
        assert!(last <= e);
        last = e;
    });
}
