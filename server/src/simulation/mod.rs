pub mod entity;
pub mod entity_ai;
pub mod physics;

use super::*;
use entity::{Entity, WishAngVel, WishLinVel};
use entity_ai::EntityAi;
use physics::*;
use rapier2d::na::{self, Rotation2, Vector2};
use rapier2d::prelude::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
type Entities = IndexMap<EntityId, Entity, RandomState>;

pub const DT: f32 = 1.0 / 10.0;
pub const DT_MS: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(u64);

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub tick: u64,
    pub half_size: f32,
    rng: SimRng,
    pub physics: Physics,

    next_entity_id: EntityId,
    pub entities: Entities,
    pub ais: IndexMap<EntityId, EntityAi, RandomState>,
}
impl Battlescape {
    pub fn new() -> Self {
        Self {
            rng: SimRng::from_entropy(),
            tick: 0,
            half_size: 100.0,
            physics: Default::default(),
            next_entity_id: EntityId(0),
            entities: Default::default(),
            ais: Default::default(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        // Afaik this can not fail.
        serde_json::to_vec(&self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn step(&mut self) {
        self.tick += 1;

        // Update ais.
        let mut remove: Vec<usize> = Vec::new();
        for (ai_idx, (entity_id, ai)) in self.ais.iter_mut().enumerate() {
            if let Some(entity_idx) = self.entities.get_index_of(entity_id) {
                if ai.update(entity_idx, &mut self.entities, &mut self.physics) {
                    remove.push(ai_idx);
                }
            }
        }
        for ai_idx in remove.into_iter().rev() {
            self.ais.swap_remove_index(ai_idx);
        }

        // Update entities.
        let mut remove: Vec<usize> = Vec::new();
        for (entity_idx, entity) in self.entities.values_mut().enumerate() {
            if entity.update(&mut self.physics) {
                remove.push(entity_idx);
            }
        }
        for entity_idx in remove.into_iter().rev() {
            if let Some((entity_id, entity)) = self.entities.swap_remove_index(entity_idx) {
                // TODO: Do something with the destroyed entity?
                self.ais.swap_remove(&entity_id);
            };
        }

        self.physics.step();
        // TODO: Handle physic events.
    }

    fn spawn_entity(&mut self) {
        todo!()
    }

    fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(entity) = self.entities.swap_remove(&entity_id) {
            // TODO:
        }
    }
}

// ###################################################################
// ############################ PACKET ###############################
// ###################################################################

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    time: f64,
    ships: Vec<ShipPacket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipPacket {
    ship_id: u64,
    ship_data_id: u32,
    position: [f32; 2],
    rotation: f32,
}

// ###################################################################
// ############################ DATA #################################
// ###################################################################

// static mut DATA: Data = Data::new();
// pub struct Data {
//     ships: Vec<ShipData>,
// }
// impl Data {
//     const fn new() -> Self {
//         Self { ships: Vec::new() }
//     }

//     pub fn data() -> &'static Self {
//         unsafe { &DATA }
//     }
// }
// struct ShipData {
//     pub id: u32,

//     pub radius: f32,

//     linear_acceleration: f32,
//     max_linear_velocity: f32,

//     angular_acceleration: f32,
//     max_angular_velocity: f32,
// }

// fn apply_wish_angular_movement(
//     time: Res<Time>,
//     mut query: Query<(
//         &mut AngularMovement,
//         &WishAngularMovement,
//         &Position,
//         &Rotation,
//     )>,
// ) {
//     query.for_each_mut(|(mut angular, wish_angular, position, rotation)| {
//         match wish_angular {
//             WishAngularMovement::Keep => {
//                 if angular.velocity.abs() > angular.max_velocity {
//                     angular.velocity = integrate_angular_velocity(
//                         angular.velocity,
//                         angular.velocity.signum() * angular.max_velocity,
//                         angular.acceleration,
//                         time.dt,
//                     );
//                 }
//             }
//             WishAngularMovement::Stop => {
//                 angular.velocity = integrate_angular_velocity(
//                     angular.velocity,
//                     0.0,
//                     angular.acceleration,
//                     time.dt,
//                 );
//             }
//             WishAngularMovement::AimSmooth(aim_to) => {
//                 // TODO: May need to rotate this.
//                 let offset = angle_to(rotation.0, *aim_to - position.0);
//                 let wish_dir = offset.signum();
//                 let mut close_smooth = offset.abs().min(0.2) / 0.2;
//                 close_smooth *= close_smooth * close_smooth;

//                 if wish_dir == angular.velocity.signum() {
//                     let time_to_target = (offset / angular.velocity).abs();
//                     let time_to_stop = (angular.velocity / (angular.acceleration)).abs();
//                     if time_to_target < time_to_stop {
//                         close_smooth *= -1.0;
//                     }
//                 }

//                 angular.velocity = integrate_angular_velocity(
//                     angular.velocity,
//                     wish_dir * angular.max_velocity * close_smooth,
//                     angular.acceleration,
//                     time.dt,
//                 );
//             }
//             WishAngularMovement::AimOverShoot(aim_to) => {
//                 // TODO: May need to rotate this.
//                 let wish_dir = angle_to(rotation.0, *aim_to - position.0).signum();
//                 angular.velocity = integrate_angular_velocity(
//                     angular.velocity,
//                     wish_dir * angular.max_velocity,
//                     angular.acceleration,
//                     time.dt,
//                 );
//             }
//             WishAngularMovement::Force(force) => {
//                 angular.velocity = integrate_angular_velocity(
//                     angular.velocity,
//                     force * angular.max_velocity,
//                     angular.acceleration,
//                     time.dt,
//                 );
//             }
//         }
//     });
// }

// fn apply_wish_linear_movement(
//     time: Res<Time>,
//     mut query: Query<(
//         &mut LinearMovement,
//         &WishLinearMovement,
//         &Position,
//         &Rotation,
//     )>,
// ) {
//     query.for_each_mut(
//         |(mut linear_movement, wish_linear_movement, position, rotation)| match wish_linear_movement
//         {
//             WishLinearMovement::Keep => {
//                 if linear_movement.velocity.length_squared()
//                     > linear_movement.max_velocity * linear_movement.max_velocity
//                 {
//                     linear_movement.velocity = integrate_linear_velocity(
//                         linear_movement.velocity,
//                         linear_movement.velocity.normalize_or_zero() * linear_movement.max_velocity,
//                         linear_movement.acceleration,
//                         time.dt,
//                     );
//                 }
//             }
//             WishLinearMovement::Stop => {
//                 linear_movement.velocity = integrate_linear_velocity(
//                     linear_movement.velocity,
//                     Vec2::ZERO,
//                     linear_movement.acceleration,
//                     time.dt,
//                 );
//             }
//             WishLinearMovement::PositionSmooth(target) => {
//                 linear_movement.velocity = integrate_linear_velocity(
//                     linear_movement.velocity,
//                     (*target - position.0).clamp_length_max(linear_movement.max_velocity),
//                     linear_movement.acceleration,
//                     time.dt,
//                 );
//             }
//             WishLinearMovement::PositionOverShoot(target) => {
//                 linear_movement.velocity = integrate_linear_velocity(
//                     linear_movement.velocity,
//                     (*target - position.0)
//                         .try_normalize()
//                         .unwrap_or(Vec2::NEG_Y)
//                         * linear_movement.max_velocity,
//                     linear_movement.acceleration,
//                     time.dt,
//                 );
//             }
//             WishLinearMovement::ForceAbsolute(force) => {
//                 linear_movement.velocity = integrate_linear_velocity(
//                     linear_movement.velocity,
//                     *force * linear_movement.max_velocity,
//                     linear_movement.acceleration,
//                     time.dt,
//                 );
//             }
//             WishLinearMovement::ForceRelative(force) => {
//                 linear_movement.velocity = integrate_linear_velocity(
//                     linear_movement.velocity,
//                     Vec2::from_angle(rotation.0).rotate(*force) * linear_movement.max_velocity,
//                     linear_movement.acceleration,
//                     time.dt,
//                 );
//             }
//         },
//     );
// }
