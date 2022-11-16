mod apply_commands;

use super::*;

impl Simulation {
    pub fn _step(&mut self, cmds: &[Command]) -> (RenderEvents, SimulationEvents) {
        self.tick += 1;

        let mut render_events = RenderEvents::default();
        let mut simulation_events = SimulationEvents::default();

        self.apply_commands(cmds, &mut render_events);

        self.ship_movement();

        // self.debug_spawn_ships(&mut ship_spawn_queue);

        self.physics.step();
        // TODO: Handle physic events.

        // self.process_ship_spawn_queue(ship_spawn_queue, &mut render_events);
    
        (render_events, simulation_events)
    }
}

impl Simulation {
    fn ship_movement(&mut self) {
        for ship in self.ships.values_mut() {
            let rb = &mut self.physics.bodies[ship.rb];

            let pos = *rb.position();
            let linvel: na::Vector2<f32> = *rb.linvel();
            

            if let Some(wish_pos) = ship.wish_pos {
                
            } else {
                
            }

            let angvel = rb.angvel();
            let wish_angvel_change = if let Some(wish_rot) = ship.wish_rot {
                let wish_angvel_change = wish_rot - pos.rotation.angle();
                // if ComplexField::abs(angvel) > ship.mobility.max_angular_velocity &&
                // ComplexField::signum(self)
                0.0
                
            } else {
                // Try to cancel our angvel.
                -angvel
            };
            let angvel_change = RealField::clamp(
                wish_angvel_change,
                -ship.mobility.angular_acceleration,
                ship.mobility.angular_acceleration
            );
            rb.set_angvel(angvel + angvel_change, true);

            let mut target_vel: na::Vector2<f32> = na::Vector2::zeros();
            let mut target_rot = pos.rotation.angle();
            let mut angvel_change = 0.0f32;

            // if let Some(client_id) = ship.contol {
            //     let inputs = self.clients.get(&client_id).unwrap().last_inputs;
                
            //     // Velocity
            //     if !inputs.stop {
            //         if inputs.wish_dir.magnitude_squared() < 0.01 {
            //             // Just keep our linvel unless we are above our limit.
            //             target_vel = linvel.cap_magnitude(ship.mobility.max_linear_velocity);
            //         } else {
            //             target_vel = inputs.wish_dir * ship.mobility.max_linear_velocity;
            //             if inputs.wish_dir_relative {
            //                 target_vel = pos.rotation.transform_vector(&target_vel);
            //             }
            //         }
            //     }

            //     // Rotation
            //     match inputs.wish_rot {
            //         WishRot::Force(force) => {
            //             target_rot = pos.rotation.angle() + force;
            //             // if na::ComplexField::abs(force) < 0.01 {
            //             //     // Slow down to reach 0 angvel without overshoot.
            //             //     angvel_change = na::RealField::min(
            //             //         ship.mobility.angular_acceleration,
            //             //         na::ComplexField::abs(angvel),
            //             //     ) * -na::ComplexField::signum(angvel);
            //             // } else {
            //             //     // Accelerate to reach max_angular_velocity without overshoot.
            //             //     angvel_change = na::RealField::clamp(
            //             //         force * ship.mobility.angular_acceleration,
            //             //         -ship.mobility.max_angular_velocity - angvel,
            //             //         ship.mobility.max_angular_velocity - angvel,
            //             //     );
            //             // }
            //         }
            //         WishRot::Toward(point) => {
            //             target_rot = (point - pos.translation.vector).angle(&na::vector![0.0, 1.0]);
            //             if ship.fleet_id.0 == 0 {
            //                 log::debug!("{:.3}", point);
            //             }
            //         }
            //     }
            // }

            target_vel = target_vel.cap_magnitude(ship.mobility.max_linear_velocity);
            let linvel_change =
                (target_vel - linvel).cap_magnitude(ship.mobility.linear_acceleration);
            rb.set_linvel(linvel + linvel_change, false);
            


            let angvel_change = na::RealField::clamp(
                target_rot - pos.rotation.angle(),
                -ship.mobility.angular_acceleration,
                ship.mobility.angular_acceleration
            );
            rb.set_angvel(angvel + angvel_change, false);
        }
    }

    // fn process_ship_spawn_queue(&mut self, ship_spawn_queue: ShipSpawnQueue, render_events: &mut BattlescapeEvents) {
    //     for (fleet_id, index) in ship_spawn_queue {
    //         let (team, ship_data_id, ship_id) = if let Some(fleet) = self.fleets.get_mut(&fleet_id)
    //         {
    //             if let Some(ship_id) = fleet.available_ships.remove(&index) {
    //                 (
    //                     fleet.team,
    //                     fleet.original_fleet.ships[index].ship_data_id,
    //                     ship_id,
    //                 )
    //             } else {
    //                 log::warn!("Ship {:?}:{} is not available. Ignoring", fleet_id, index);
    //                 continue;
    //             }
    //         } else {
    //             log::warn!("{:?} does not exist. Ignoring", fleet_id);
    //             continue;
    //         };

    //         let ship_data = ship_data_id.data();
    //         let group_ignore = self.physics.new_group_ignore();
    //         let spawn_position = na::Isometry2::new(
    //             self.ship_spawn_position(team),
    //             self.ship_spawn_rotation(team),
    //         );

    //         let rb = RigidBodyBuilder::dynamic()
    //             .position(spawn_position)
    //             .user_data(UserData::build(
    //                 team,
    //                 group_ignore,
    //                 GenericId::ShipId(ship_id),
    //                 false,
    //             ))
    //             .can_sleep(false)
    //             .build();
    //         let parrent_rb = self.physics.bodies.insert(rb);

    //         // Add hulls.
    //         let main_hull = self.add_hull(
    //             ship_data.main_hull,
    //             team,
    //             group_ignore,
    //             parrent_rb,
    //             GROUPS_SHIP,
    //         );
    //         let auxiliary_hulls: AuxiliaryHulls = ship_data
    //             .auxiliary_hulls
    //             .iter()
    //             .map(|&hull_data_id| {
    //                 self.add_hull(hull_data_id, team, group_ignore, parrent_rb, GROUPS_SHIP)
    //             })
    //             .collect();

    //         self.ships.insert(
    //             ship_id,
    //             BattlescapeShip {
    //                 fleet_id,
    //                 index,
    //                 contol: None,
    //                 ship_data_id,
    //                 rb: parrent_rb,
    //                 mobility: ship_data.mobility,
    //                 main_hull,
    //                 auxiliary_hulls,
    //             },
    //         );

    //         // Add event.
    //         render_events.add_ship.push(ship_id);
    //     }
    // }

    // fn add_hull(
    //     &mut self,
    //     hull_data_id: HullDataId,
    //     team: u32,
    //     group_ignore: u32,
    //     parrent_rb: RigidBodyHandle,
    //     groups: InteractionGroups,
    // ) -> HullId {
    //     let hull_data = hull_data_id.data();
    //     let hull_id = self.new_hull_id();
    //     let user_data =
    //         UserData::build(team, group_ignore, GenericId::from_hull_id(hull_id), false);
    //     let coll = build_hull_collider(hull_data_id, groups, user_data);
    //     let coll_handle = self.physics.insert_collider(parrent_rb, coll);
    //     self.hulls.insert(
    //         hull_id,
    //         Hull {
    //             hull_data_id,
    //             current_defence: hull_data.defence,
    //             collider: coll_handle,
    //         },
    //     );
    //     hull_id
    // }
}

// impl Battlescape {
//     #[deprecated]
//     fn debug_spawn_ships(&mut self, ship_spawn_queue: &mut ShipSpawnQueue) {
//         // rapier2d::na::ComplexField::
//         for i in 0..self.fleets.len() {
//             let fleet_id = FleetId(i as u64);
//             let fleet = self.fleets.get(&fleet_id).unwrap();
//             for ship_index in fleet.available_ships.keys() {
//                 ship_spawn_queue.insert((fleet_id, *ship_index));
//                 break;
//             }
//         }
//     }
// }