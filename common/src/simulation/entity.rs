use super::*;

/// `[0..1]` relative to armor_hp_max.
///
/// Minimum 3x3.
type ArmorCells = SmallVec<[u8; 16]>;

/// Returns if the entity should be retained.
// pub fn update_entity_retain(sim: &mut Simulation, entity_idx: usize) -> bool {
//     // Check if target is still valid.
//     let target_idx = if let Some(target) = sim.hulls[entity_idx].target {
//         let target_idx = sim.hulls.get_index_of(&target);

//         if target_idx.is_none() {
//             sim.hulls[entity_idx].target = None;
//         }

//         target_idx
//     } else {
//         None
//     };

//     // Update modifiers.
//     let mut modifier_idx = 0;
//     while modifier_idx < sim.hulls[entity_idx].modifiers.len() {
//         let mut modifier = std::mem::take(&mut sim.hulls[entity_idx].modifiers[modifier_idx]);

//         match &mut modifier {
//             Modifier::Nothing => {}
//             Modifier::AiSeek => {
//                 if let Some(target_idx) = target_idx {
//                     let target = *sim.physics.body(sim.hulls[target_idx].rb).translation();
//                     sim.hulls[entity_idx].wish_angvel = WishAngVel::AimSmooth(target);
//                 }
//             }
//             Modifier::AiShip => {
//                 // TODO
//             }
//         }

//         if let Modifier::Nothing = modifier {
//             sim.hulls[entity_idx].modifiers.swap_remove(modifier_idx);
//         } else {
//             sim.hulls[entity_idx].modifiers[modifier_idx] = modifier;
//             modifier_idx += 1;
//         }
//     }

//     let entity = &mut sim.hulls[entity_idx];
//     let rb = sim.physics.body_mut(entity.rb);
//     let angvel = rb.angvel();
//     let linvel = *rb.linvel();

//     let wish_angvel = match entity.wish_angvel {
//         WishAngVel::None => angvel,
//         WishAngVel::Keep => angvel.clamp(-entity.max_angular_velocity, entity.max_angular_velocity),
//         WishAngVel::Stop => 0.0,
//         WishAngVel::AimSmooth(aim_to) => {
//             // TODO: May need to rotate this.
//             // aim_to.angle(other)
//             // let offset = Vec2::from_angle(angle).angle_between(to);

//             // let offset = angle_to(rotation.0, *aim_to - position.0);
//             // let wish_dir = offset.signum();
//             // let mut close_smooth = offset.abs().min(0.2) / 0.2;
//             // close_smooth *= close_smooth * close_smooth;

//             // if wish_dir == angular.velocity.signum() {
//             //     let time_to_target = (offset / angular.velocity).abs();
//             //     let time_to_stop = (angular.velocity / (angular.acceleration)).abs();
//             //     if time_to_target < time_to_stop {
//             //         close_smooth *= -1.0;
//             //     }
//             // }

//             // angular.velocity = integrate_angular_velocity(
//             //     angular.velocity,
//             //     wish_dir * angular.max_velocity * close_smooth,
//             //     angular.acceleration,
//             //     time.dt,
//             // );
//             0.0
//         }
//         WishAngVel::Force(force) => force * entity.max_angular_velocity,
//     };

//     let wish_linvel = match entity.wish_linvel {
//         WishLinVel::None => linvel,
//         WishLinVel::Keep => linvel.cap_magnitude(entity.max_linear_velocity),
//         WishLinVel::Cancel => vector![0.0, 0.0],
//         WishLinVel::PositionSmooth(position) => {
//             let to_position = position - rb.translation();
//             if to_position.magnitude_squared() < 0.01 {
//                 vector![0.0, 0.0]
//             } else {
//                 to_position.cap_magnitude(entity.max_linear_velocity)
//             }
//         }
//         WishLinVel::PositionOvershoot(position) => {
//             (position - rb.translation())
//                 .try_normalize(0.01)
//                 .unwrap_or(vector![0.0, -1.0])
//                 * entity.max_linear_velocity
//         }
//         WishLinVel::ForceAbsolute(force) => force * entity.max_linear_velocity,
//         WishLinVel::ForceRelative(force) => {
//             rb.rotation().transform_vector(&force) * entity.max_linear_velocity
//         }
//     };

//     let wake_up = wish_angvel != angvel || wish_linvel != linvel;
//     // todo dt is fixed
//     rb.set_angvel(
//         angvel
//             + (wish_angvel - angvel).clamp(
//                 -entity.angular_acceleration * DT.as_secs_f32(),
//                 entity.angular_acceleration * DT.as_secs_f32(),
//             ),
//         wake_up,
//     );
//     rb.set_linvel(
//         linvel + (wish_linvel - linvel).cap_magnitude(entity.linear_acceleration),
//         wake_up,
//     );

//     entity.hull > 0.0
// }

#[test]
fn test_rotation() {
    let a_translation = vector![100.0f32, 200.0];
    let a_rotation = f32::to_radians(35.0);
    let mut body = RigidBodyBuilder::dynamic().build();
    body.set_translation(a_translation, true);
    body.set_rotation(na::UnitComplex::new(a_rotation), true);

    let b_translation = point![-50.0f32, 70.0];
    let b_rotation = f32::to_radians(45.0);
    let b_position = na::Isometry2::new(b_translation.coords, b_rotation);

    let target = point![350.0f32, 300.0];

    let global_translation = body.position().transform_point(&b_translation);
    let global_position = body.position() * b_position;
    let global_rotation = global_position.rotation.angle();
    let rotation_to_target =
        global_position
            .rotation
            .rotation_to(&na::UnitComplex::rotation_between(
                &vector![1.0, 0.0],
                &(target.coords - global_position.translation.vector),
            ));

    println!("{:?}", global_translation);
    println!("{:?}", global_rotation);
    println!("{:?}", rotation_to_target.angle());
}
