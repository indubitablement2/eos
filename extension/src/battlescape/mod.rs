pub mod angle_vector;
pub mod bc_client;
pub mod bc_fleet;
pub mod command;
pub mod entity;
pub mod events;
pub mod mode;
pub mod physics;

use super::*;
use angle_vector::VectorAngle;
use godot::prelude::ToVariant;
use rand::prelude::*;
use rapier2d::prelude::*;

use bc_client::BattlescapeClient;
use bc_fleet::{BattlescapeFleet, FleetShipState};
use entity::ai::EntityAi;
use entity::{Entity, WishAngVel, WishLinVel};
use events::BattlescapeEventHandler;
use physics::*;

pub use self::mode::BattlescapeMode;
pub use command::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
type Entities = IndexMap<EntityId, Entity, RandomState>;
type Clients = IndexMap<ClientId, BattlescapeClient, RandomState>;
type Fleets = IndexMap<FleetId, BattlescapeFleet, RandomState>;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(u32);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattlescapeStateInit {
    pub seed: u64,
    pub mode: BattlescapeMode,
}

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub tick: u64,
    pub mode: BattlescapeMode,
    pub half_size: f32,
    /// Battle will end when timeout reach 0.
    pub end_timeout: u32,
    rng: SimRng,
    pub physics: Physics,

    pub team_num_active_ship: AHashMap<u32, u32>,
    pub fleets: Fleets,
    pub clients: Clients,

    next_entity_id: EntityId,
    pub entities: Entities,
    pub ais: IndexMap<EntityId, EntityAi, RandomState>,
}
impl Battlescape {
    /// Amount of tick with active ships from only one team before battle is over.
    pub const END_TIMEOUT: u32 = (1.0 / DT) as u32 * 10;
    /// Amount of tick before end timeout may start.
    pub const END_TIMEOUT_START_TICK: u64 = (1.0 / DT) as u64 * 50;

    pub const SPAWN_OFFSET: f32 = 10.0;

    pub fn new(state_init: BattlescapeStateInit) -> Self {
        Self {
            rng: SimRng::seed_from_u64(state_init.seed),
            tick: Default::default(),
            mode: state_init.mode,
            half_size: 100.0,
            end_timeout: Self::END_TIMEOUT,
            physics: Default::default(),
            team_num_active_ship: Default::default(),
            fleets: Default::default(),
            clients: Default::default(),
            next_entity_id: EntityId(0),
            entities: Default::default(),
            ais: Default::default(),
        }
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        for entity in self.entities.values_mut() {
            entity.pre_serialize();
        }

        // Afaik this can not fail.
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }

    pub fn post_deserialize_prepare(&mut self, bc_ptr: i64) {
        for (entity, entity_idx) in self.entities.values_mut().zip(0i64..) {
            entity.post_deserialize_prepare(bc_ptr.to_variant(), entity_idx.to_variant());
        }

        for entity in self.entities.values_mut() {
            entity.post_deserialize_post_prepare();
        }
    }

    /// Take the cmds for the tick `self.tick + 1`.
    pub fn step(&mut self, cmds: &Commands, events: &mut impl BattlescapeEventHandler) {
        if self.end_timout() {
            events.battle_over(&self);
            return;
        }

        self.tick += 1;

        cmds.apply(self, events);

        self.ai();
        self.movement();

        self.physics.step();
        // TODO: Handle physic events.

        if self.end_timeout == 0 {
            events.battle_over(&self);
        }
    }

    fn end_timout(&mut self) -> bool {
        if self.tick < Self::END_TIMEOUT_START_TICK {
            return false;
        }

        if self.end_timeout == 0 {
            return true;
        }

        if self
            .team_num_active_ship
            .values()
            .map(|num_active_ship| *num_active_ship > 0)
            .count()
            < 2
        {
            self.end_timeout -= 1;
        } else {
            self.end_timeout = Self::END_TIMEOUT;
        }

        false
    }

    fn ai(&mut self) {
        // Ais that want to be removed.
        let mut remove: Vec<EntityId> = Vec::new();

        for (entity_id, ai) in self.ais.iter_mut() {
            if ai.remove() {
                remove.push(*entity_id);
            } else if let Some((entity_index, _, _)) = self.entities.get_full(entity_id) {
                ai.update(
                    entity_index,
                    &mut self.entities,
                    &mut self.physics,
                    &mut self.clients,
                    &mut self.fleets,
                );
            } else {
                // No matching entity.
                remove.push(*entity_id);
            }
        }

        for entity_id in remove {
            self.ais.swap_remove(&entity_id);
        }
    }

    fn movement(&mut self) {
        // for e in self.entities.values_mut() {
        //     let rb = &mut self.physics.bodies[e.rb];

        //     // TODO: Does this work?
        //     fn apply_wish_angvel(
        //         angle_change: f32,
        //         rb: &mut RigidBody,
        //         max_angular_velocity: f32,
        //         angular_acceleration: f32,
        //         wake_up: bool,
        //     ) {
        //         let wish_angvel =
        //             RealField::clamp(angle_change, -max_angular_velocity, max_angular_velocity);
        //         let vel_change = RealField::clamp(
        //             wish_angvel - rb.angvel(),
        //             -angular_acceleration * DT,
        //             angular_acceleration * DT,
        //         );
        //         rb.set_angvel(rb.angvel() + vel_change, wake_up);
        //     }

        //     // Angvel
        //     match e.wish_angvel {
        //         WishAngVel::Keep => {}
        //         WishAngVel::Cancel => {
        //             if ComplexField::abs(rb.angvel()) > 0.001 {
        //                 rb.set_angvel(
        //                     rb.angvel()
        //                         - RealField::clamp(
        //                             rb.angvel(),
        //                             -e.mobility.angular_acceleration,
        //                             e.mobility.angular_acceleration,
        //                         ),
        //                     false,
        //                 );
        //             }
        //         }
        //         WishAngVel::Aim { position } => {
        //             // TODO: Does this work?
        //             let relative_pos = position - *rb.translation();
        //             let angle_change = rb
        //                 .rotation()
        //                 .transform_vector(&na::vector![0.0, 1.0])
        //                 .angle(&relative_pos);
        //             apply_wish_angvel(
        //                 angle_change,
        //                 rb,
        //                 e.mobility.max_angular_velocity,
        //                 e.mobility.angular_acceleration,
        //                 true,
        //             );
        //         }
        //         WishAngVel::Rotation(wish_rot) => {
        //             // TODO:
        //         }
        //     }

        //     // let angvel = rb.angvel();
        //     // let wish_angvel = if let Some(wish_rot) = e.wish_rot {
        //     //     RealField::clamp(
        //     //         pos.rotation.rotation_to(&wish_rot).angle(),
        //     //         -e.mobility.max_angular_velocity,
        //     //         e.mobility.max_angular_velocity,
        //     //     )
        //     // } else {
        //     //     // Try to cancel our angvel.
        //     //     -angvel
        //     // };
        //     // let angvel = RealField::clamp(
        //     //     wish_angvel - angvel,
        //     //     -e.mobility.angular_acceleration,
        //     //     e.mobility.angular_acceleration,
        //     // );
        //     // rb.set_angvel(angvel, true);

        //     fn apply_wish_linvel(
        //         wish_vel: Vector2,
        //         rb: &mut RigidBody,
        //         linear_acceleration: f32,
        //         wake_up: bool,
        //     ) {
        //         let vel_change = (wish_vel - rb.linvel()).cap_magnitude(linear_acceleration * DT);
        //         rb.set_linvel(rb.linvel() + vel_change, wake_up);
        //     }

        //     // Linvel
        //     match e.wish_linvel {
        //         WishLinVel::Keep => {}
        //         WishLinVel::Cancel => {
        //             if rb.linvel().magnitude_squared() > 0.001 {
        //                 apply_wish_linvel(-rb.linvel(), rb, e.mobility.linear_acceleration, false);
        //                 // rb.set_linvel(
        //                 //     rb.linvel() - rb.linvel().cap_magnitude(e.mobility.linear_acceleration * DT),
        //                 //     false,
        //                 // );
        //             }
        //         }
        //         WishLinVel::Forward => {
        //             let wish_vel = rb
        //                 .rotation()
        //                 .transform_vector(&na::vector![0.0, e.mobility.max_linear_velocity]);
        //             apply_wish_linvel(wish_vel, rb, e.mobility.linear_acceleration, true);
        //         }
        //         WishLinVel::Position { position } => {
        //             // TODO:
        //         }
        //         WishLinVel::PositionOvershot { position } => {
        //             // TODO:
        //         }
        //         WishLinVel::Absolute { angle, strenght } => {
        //             let wish_vel = angle.transform_vector(&na::vector![
        //                 0.0,
        //                 e.mobility.max_linear_velocity * strenght
        //             ]);
        //             apply_wish_linvel(wish_vel, rb, e.mobility.linear_acceleration, true);
        //         }
        //         WishLinVel::Relative { angle, strenght } => {
        //             let wish_vel =
        //                 rb.rotation()
        //                     .transform_vector(&angle.transform_vector(&na::vector![
        //                         0.0,
        //                         e.mobility.max_linear_velocity * strenght
        //                     ]));
        //             apply_wish_linvel(wish_vel, rb, e.mobility.linear_acceleration, true);
        //         }
        //     }

        //     // let linvel: Vector2 = *rb.linvel();
        //     // let wish_linvel = if let Some(wish_pos) = e.wish_pos {
        //     //     let linvel_magnitude = linvel.magnitude();
        //     //     let time_to_break = linvel_magnitude / (e.mobility.linear_acceleration * 1.0);

        //     //     let relative_target = wish_pos - pos.translation.vector;

        //     //     let wish_vel = relative_target - linvel * time_to_break;
        //     //     wish_vel.cap_magnitude(e.mobility.max_linear_velocity)
        //     // } else {
        //     //     na::Vector2::zeros()
        //     // };
        //     // let linvel = (wish_linvel - linvel).cap_magnitude(e.mobility.linear_acceleration);
        //     // rb.set_linvel(linvel, true);
        // }
    }

    fn add_fleet_ship(
        &mut self,
        fleet_id: FleetId,
        index: usize,
        prefered_spawn_point: usize,
        events: &mut impl BattlescapeEventHandler,
    ) {
        if let Some(fleet) = self.fleets.get_mut(&fleet_id) {
            let entity_id = self.next_entity_id;

            let spawn_points = self.mode.spawn_points(fleet.team);
            let spawn_point = spawn_points
                .get(prefered_spawn_point)
                .unwrap_or_else(|| &spawn_points[0]);

            if let Some(entity) = fleet.try_spawn(
                index,
                spawn_point,
                self.half_size,
                entity_id,
                &mut self.physics,
            ) {
                self.next_entity_id.0 += 1;
                *self.team_num_active_ship.entry(fleet.team).or_default() += 1;
                let i = self.entities.insert_full(entity_id, entity).0;
                events.entity_added(entity_id, &self.entities[i]);
            }
        }
    }

    fn remove_entity(&mut self, entity_id: EntityId, events: &mut impl BattlescapeEventHandler) {
        if let Some(entity) = self.entities.swap_remove(&entity_id) {
            // Handle if this is a ship from a fleet.
            if let Some((fleet_id, index)) = entity.fleet_ship {
                let fleet = self.fleets.get_mut(&fleet_id).unwrap();
                let fleet_ship = &mut fleet.ships[index];

                if let Some(result) = entity.result() {
                    fleet_ship.state = FleetShipState::Removed(result);
                } else {
                    // Ship destroyed.
                    fleet_ship.state = FleetShipState::Destroyed;
                    events.ship_destroyed(fleet_id, index);
                }

                *self.team_num_active_ship.get_mut(&entity.team).unwrap() -= 1;
            }

            events.entity_removed(entity_id, entity);
        }
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}