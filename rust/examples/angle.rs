use glam::{vec2, Vec2};
use rand::random;

fn main() {
    let rot = ["l", "tl", "t", "tr", "r", "br", "b", "bl"];

    let v: Vec<Vec2> = (0..6)
        .into_iter()
        .map(|_| vec2(random::<f32>().mul_add(2.0, -1.0), random::<f32>().mul_add(2.0, -1.0)))
        .collect();

    v.iter().for_each(|v2| {
        println!("{:?}", v2);

        let mut a = v2.y.atan2(v2.x) + std::f32::consts::PI;

        println!("{}, {}", &a, a.to_degrees());

        a = a.mul_add(8.0 / std::f32::consts::TAU, 0.5);

        println!("{}", &a);

        let id = unsafe { a.to_int_unchecked::<u32>() % 8 };

        println!("{}, {}\n", &id, rot[id as usize]);
    });
}
