use rand::prelude::*;
// use std::{thread::sleep, time::Duration};

// average & highest

fn main() {
    let mut bu = 0.0f32;
    let mut highest_bu = 0.0f32;

    for _ in 0..1000000 {
        let r: u32 = random();

        let num_sent = (r % 10).saturating_sub(5);

        bu += num_sent as f32;
        bu *= 0.14;

        if bu > highest_bu {
            highest_bu = bu
        }
    }

    println!("highest_bu: {:.5}", highest_bu);
}
