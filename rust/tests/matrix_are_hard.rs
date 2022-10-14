// use glam::{vec2, Mat3};

// #[test]
// fn matrix_are_hard() {
//     let gv = vec2(50.0, 33.3);
//     let gmat = Mat3::from_scale_angle_translation(vec2(1.0, 1.0), 1.0, gv);

//     println!("{:?}", &gmat);

//     println!("{:?}", &gmat.x_axis.x);
//     println!("{:?}", &gmat.y_axis.x);
//     println!("{:?}", &gmat.x_axis.z); // ? 0.0
//     println!("{:?}", &gmat.z_axis.x);
//     println!("{:?}", &gmat.x_axis.y);
//     println!("{:?}", &gmat.y_axis.y);
//     println!("{:?}", &gmat.y_axis.z); // ? 0.0
//     println!("{:?}", &gmat.z_axis.y);

//     // let iso = Isometry2::new(vector![50.0, 33.3], 1.22f32);
//     // let mat = iso.to_matrix();

//     // println!("{:?}", &mat.data);

//     // println!("{:?}", &mat[(0, 0)]);
//     // println!("{:?}", &mat[(0, 1)]);
//     // println!("{:?}", &mat[(2, 0)]); // 0.0
//     // println!("{:?}", &mat[(0, 2)]);
//     // println!("{:?}", &mat[(1, 0)]);
//     // println!("{:?}", &mat[(1, 1)]);
//     // println!("{:?}", &mat[(2, 1)]); // 0.0
//     // println!("{:?}", &mat[(1, 2)]);
// }

// // Expected:
// // [[0.34364575, 0.9390994, 0.0], [-0.9390994, 0.34364575, 0.0], [50.0, 33.3, 1.0]]
// // 0.34364575
// // -0.9390994
// // 0.0
// // 50.0
// // 0.9390994
// // 0.34364575
// // 0.0
// // 33.3
