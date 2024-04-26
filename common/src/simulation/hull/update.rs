use super::*;

impl Hull {
    pub fn on_update(
        &mut self,
        current: HullId,
        hulls: &mut Hulls,
        clients: &mut Clients,
    ) -> Option<RemoveReason> {
        // Check that target is exist.
        if let Some(target) = self.target {
            if !hulls.contains(target) {
                self.target = None;
            }
        }

        let mut remove_reason = None;

        self.modifiers.retain(|modifier| match modifier {
            Modifier::RemoveThis => false,
        });

        match self.hull_data_id.0.ai {
            HullAi::None => {}
            HullAi::Ship => {}
            HullAi::Seek => {
                if let Some(target) = self.target {
                    self.wish_angvel = WishAngVel::AimSmooth(hulls.get(target).unwrap().position);
                } else {
                    self.wish_angvel = WishAngVel::Stop;
                }
            }
        }

        self._apply_wish_angvel();
        self._apply_wish_linvel();

        // TODO: Only update clients which can see this
        for client in clients.values_mut() {
            client.hull_update(current, self);
        }

        if self.hull_relative <= 0.0 {
            Some(RemoveReason::Destroyed)
        } else {
            remove_reason
        }
    }

    fn _apply_wish_linvel(&mut self) {
        let wish_linvel = match self.wish_linvel {
            WishLinVel::None => return,
            WishLinVel::Keep => self.linvel.cap_magnitude(self.linvel_max),
            WishLinVel::Stop => Vector2::zeros(),
            WishLinVel::PositionSmooth(target) => {
                let to_pos = target - self.position;
                if to_pos.magnitude_squared() < 0.5 {
                    vector![0.0, 0.0]
                } else {
                    to_pos.cap_magnitude(self.linvel_max)
                }
            }
            WishLinVel::PositionOvershoot(target) => {
                let to_pos = target - self.position;
                to_pos.try_normalize(0.1).unwrap_or(vector![0.0, 1.0]) * self.linvel_max
            }
            WishLinVel::ForceAbsolute(force) => force.cap_magnitude(1.0) * self.linvel_max,
            WishLinVel::ForceRelative(force) => {
                self.rotation.transform_vector(&force.cap_magnitude(1.0)) * self.linvel_max
            }
        };

        let linvel_change =
            (wish_linvel - self.linvel).cap_magnitude(self.linacc * DT.as_secs_f32());
        if linvel_change.x.abs() > 0.001 || linvel_change.y.abs() > 0.001 {
            self.linvel += linvel_change;
        }

        // TODO: Thruster
    }

    fn _apply_wish_angvel(&mut self) {
        let wish_angvel: f32 = match self.wish_angvel {
            WishAngVel::None => return,
            WishAngVel::Keep => self.angvel.clamp(-self.angvel_max, self.angvel_max),
            WishAngVel::Stop => 0.0,
            WishAngVel::AimSmooth(target) => {
                let offset = (self.position - target).angle(&vector![1.0, 0.0]);
                let wish_dir = if offset < 0.0 { -1.0 } else { 1.0 };
                let angvel_dir = if self.angvel < 0.0 { -1.0 } else { 1.0 };

                let mut close_smooth = offset.abs().min(0.2) / 0.2;
                close_smooth *= close_smooth * close_smooth;

                if wish_dir == angvel_dir {
                    let time_to_target = (offset / self.angvel).abs();
                    let time_to_stop = (self.angvel / self.angacc).abs();

                    if time_to_target < time_to_stop {
                        close_smooth *= -1.0;
                    }
                }

                wish_dir * self.angvel_max * close_smooth
            }
            WishAngVel::Force(force) => force.clamp(-1.0, 1.0) * self.angvel_max,
        };

        let angacc_dt = self.angacc * DT.as_secs_f32();
        let angvel_change = wish_angvel.clamp(-angacc_dt, angacc_dt);
        if angvel_change.abs() > 0.01 {
            self.angvel += angvel_change;
        }

        // TODO: Thruster
    }
}
