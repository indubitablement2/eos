mod apply_commands;
use super::*;

impl Battlescape {
    pub fn _step(&mut self, cmds: &[BattlescapeCommand]) {
        let mut ship_spawn_queue = ShipSpawnQueue::new();

        self.apply_commands(cmds);

        self.update_client_control();
        self.ship_ai();

        self.debug_spawn_ships(&mut ship_spawn_queue);

        self.physics.step();
        // TODO: Handle physic events.

        self.process_ship_spawn_queue(ship_spawn_queue);

        self.tick += 1;
    }
}

impl Battlescape {
    fn update_client_control(&mut self) {
        for client in self.clients.values_mut() {
            if !client.active(self.tick) {
                if let Some(ship_id) = client.control.take() {
                    if let Some(ship) = self.ships.get_mut(&ship_id) {
                        ship.contol = None;
                        log::info!("Removed control from {:?} as owner is inactive", ship_id);
                    }
                }
            }
        }
    }

    fn ship_ai(&mut self) {
        for ship in self.ships.values_mut() {
            let rb = &mut self.physics.bodies[ship.rb];

            let pos = *rb.position();
            let linvel: na::Vector2<f32> = *rb.linvel();
            let angvel = rb.angvel();
            let mut target_vel: na::Vector2<f32> = na::Vector2::zeros();
            let mut angvel_change = 0.0f32;

            if let Some(client_id) = ship.contol {
                let inputs = self.clients.get(&client_id).unwrap().last_inputs;

                // Velocity
                if !inputs.stop {
                    if inputs.wish_dir.magnitude_squared() < 0.01 {
                        // Just keep our linvel unless we are above our limit.
                        target_vel = linvel.cap_magnitude(ship.mobility.max_linear_velocity);
                    } else {
                        target_vel = inputs.wish_dir * ship.mobility.max_linear_velocity;
                        if inputs.wish_dir_relative {
                            target_vel = pos.rotation.transform_vector(&target_vel);
                        }
                    }
                }

                // Rotation
                if inputs.wish_rot_absolute {
                    // TODO: abs wish rot
                } else {
                    if na::ComplexField::abs(inputs.wish_rot) < 0.01 {
                        // Slow down to reach 0 angvel without overshoot.
                        angvel_change = na::RealField::min(
                            ship.mobility.angular_acceleration,
                            na::ComplexField::abs(angvel),
                        ) * -na::ComplexField::signum(angvel);
                    } else {
                        // Accelerate to reach max_angular_velocity without overshoot.
                        angvel_change = na::RealField::clamp(
                            inputs.wish_rot * ship.mobility.angular_acceleration,
                            -ship.mobility.max_angular_velocity - angvel,
                            ship.mobility.max_angular_velocity - angvel,
                        );
                    }
                }
            }

            let linvel_change =
                (target_vel - linvel).cap_magnitude(ship.mobility.linear_acceleration);
            rb.set_linvel(linvel + linvel_change, false);
            rb.set_angvel(angvel + angvel_change, false);
        }
    }

    fn process_ship_spawn_queue(&mut self, ship_spawn_queue: ShipSpawnQueue) {
        for (fleet_id, index) in ship_spawn_queue {
            let (team, ship_data_id, ship_id) = if let Some(fleet) = self.fleets.get_mut(&fleet_id)
            {
                if let Some(ship_id) = fleet.available_ships.remove(&index) {
                    (
                        fleet.team,
                        fleet.original_fleet.ships[index].ship_data_id,
                        ship_id,
                    )
                } else {
                    log::warn!("Ship {:?}:{} is not available. Ignoring", fleet_id, index);
                    continue;
                }
            } else {
                log::warn!("{:?} does not exist. Ignoring", fleet_id);
                continue;
            };

            let ship_data = ship_data_id.data();
            let group_ignore = self.physics.new_group_ignore();
            let spawn_position = na::Isometry2::new(
                self.ship_spawn_position(team),
                self.ship_spawn_rotation(team),
            );

            let rb = RigidBodyBuilder::dynamic()
                .position(spawn_position)
                .user_data(UserData::build(
                    team,
                    group_ignore,
                    GenericId::ShipId(ship_id),
                    false,
                ))
                .can_sleep(false)
                .build();
            let parrent_rb = self.physics.bodies.insert(rb);

            // Add hulls.
            let main_hull = self.add_hull(
                ship_data.main_hull,
                team,
                group_ignore,
                parrent_rb,
                GROUPS_SHIP,
            );
            let auxiliary_hulls: AuxiliaryHulls = ship_data
                .auxiliary_hulls
                .iter()
                .map(|&hull_data_id| {
                    self.add_hull(hull_data_id, team, group_ignore, parrent_rb, GROUPS_SHIP)
                })
                .collect();

            self.ships.insert(
                ship_id,
                BattlescapeShip {
                    fleet_id,
                    index,
                    contol: None,
                    ship_data_id,
                    rb: parrent_rb,
                    mobility: ship_data.mobility,
                    main_hull,
                    auxiliary_hulls,
                },
            );
        }
    }

    fn add_hull(
        &mut self,
        hull_data_id: HullDataId,
        team: u32,
        group_ignore: u32,
        parrent_rb: RigidBodyHandle,
        groups: InteractionGroups,
    ) -> HullId {
        let hull_data = hull_data_id.data();
        let hull_id = self.new_hull_id();
        let user_data =
            UserData::build(team, group_ignore, GenericId::from_hull_id(hull_id), false);
        let coll = build_hull_collider(hull_data_id, groups, user_data);
        let coll_handle = self.physics.insert_collider(parrent_rb, coll);
        self.hulls.insert(
            hull_id,
            Hull {
                hull_data_id,
                current_defence: hull_data.defence,
                collider: coll_handle,
            },
        );
        hull_id
    }
}

impl Battlescape {
    #[deprecated]
    fn debug_spawn_ships(&mut self, ship_spawn_queue: &mut ShipSpawnQueue) {
        for (fleet_id, fleet) in self.fleets.iter() {
            for ship_index in fleet.available_ships.keys() {
                ship_spawn_queue.insert((*fleet_id, *ship_index));
                break;
            }
        }
    }
}
