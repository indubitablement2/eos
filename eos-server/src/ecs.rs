// TODO: DELETE

use std::{
    thread::{self, sleep},
    time::Duration,
};

use bevy_ecs::prelude::*;

struct Comp1(u32);
struct Comp2(u32);

pub fn start() {
    thread::spawn(|| {
        let mut world = World::default();

        world.spawn().insert(Comp1(5));
        world.spawn().insert(Comp1(0));

        let mut schedule = Schedule::default();

        schedule.add_stage("first", SystemStage::single_threaded());
        schedule.add_stage_after("first", "last", SystemStage::single_threaded());

        schedule.add_system_to_stage("first", dummy.system());
        schedule.add_system_to_stage("first", print_all.system());

        loop {
            schedule.run_once(&mut world);
            sleep(Duration::from_millis(250));
        }
    });
}

fn dummy(mut cmd: Commands, query: Query<(Entity, &mut Comp1)>) {
    query.for_each_mut(|(e, mut c)| {
        c.0 += 1;
        if c.0 > 10 {
            cmd.entity(e).remove::<Comp1>().insert(Comp2(0));
        }
    });
}

fn print_all(query: Query<(Entity, Option<&Comp1>, Option<&Comp2>)>) {
    query.for_each(|(e, c1, c2)| {
        if let Some(c1) = c1 {
            println!("{:?}, c1 = {}", e, c1.0);
        }

        if let Some(c2) = c2 {
            println!("{:?}, c2 = {}", e, c2.0);
        }
    });
}
