use nalgebra::Isometry2;
use rapier2d::na;

#[test]
fn matrix_are_hard() {
    let iso = Isometry2::new(na::vector![50.0, 33.3], 1.22f32);
    let mat = iso.to_matrix();

    println!("{:?}", &mat.data);

    println!("{:?}", &mat[(0, 0)]);
    println!("{:?}", &mat[(0, 1)]);
    println!("{:?}", &mat[(2, 0)]); // 0.0
    println!("{:?}", &mat[(0, 2)]);
    println!("{:?}", &mat[(1, 0)]);
    println!("{:?}", &mat[(1, 1)]);
    println!("{:?}", &mat[(2, 1)]); // 0.0
    println!("{:?}", &mat[(1, 2)]);
}
