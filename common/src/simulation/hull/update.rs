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

        if self.hull_relative <= 0.0 {
            Some(RemoveReason::Destroyed)
        } else {
            remove_reason
        }
    }

    fn _apply_wish_linvel(&mut self) {
        let wish_linvel = match self.wish_linvel {
            WishLinVel::None => return,
            WishLinVel::Keep => self.linvel.clamp_length_max(self.linvel_max()),
            WishLinVel::Stop => Vec2::ZERO,
            WishLinVel::PositionSmooth(target) => {
                let to_pos = target - self.position;
                if to_pos.length_squared() < 0.5 {
                    Vec2::ZERO
                } else {
                    to_pos.clamp_length_max(self.linvel_max())
                }
            }
            WishLinVel::PositionOvershoot(target) => {
                let to_pos = target - self.position;
                to_pos.try_normalize().unwrap_or(Vec2::Y) * self.linvel_max()
            }
            WishLinVel::ForceAbsolute(force) => force.clamp_length_max(1.0) * self.linvel_max(),
            WishLinVel::ForceRelative(force) => {
                force
                    .clamp_length_max(1.0)
                    .rotate(Vec2::from_angle(self.rotation))
                    * self.linvel_max()
            }
        };

        let linvel_change =
            (wish_linvel - self.linvel).clamp_length_max(self.linacc() * DT.as_secs_f32());
        if linvel_change.x.abs() > 0.001 || linvel_change.y.abs() > 0.001 {
            self.linvel += linvel_change;
        }

        // TODO: Thruster
    }

    fn _apply_wish_angvel(&mut self) {
        let wish_angvel: f32 = match self.wish_angvel {
            WishAngVel::None => return,
            WishAngVel::Keep => self.angvel.clamp(-self.angvel_max(), self.angvel_max()),
            WishAngVel::Stop => 0.0,
            WishAngVel::AimSmooth(target) => {
                let mut offset = (self.position - target).to_angle();
                if !f32::is_normal(offset) {
                    offset = 0.0;
                }
                let wish_dir = if offset < 0.0 { -1.0 } else { 1.0 };
                let angvel_dir = if self.angvel < 0.0 { -1.0 } else { 1.0 };

                let mut close_smooth = offset.abs().min(0.2) / 0.2;
                close_smooth *= close_smooth * close_smooth;

                if wish_dir == angvel_dir {
                    let time_to_target = (offset / self.angvel).abs();
                    let time_to_stop = (self.angvel / self.angacc()).abs();

                    if time_to_target < time_to_stop {
                        close_smooth *= -1.0;
                    }
                }

                wish_dir * self.angvel_max() * close_smooth
            }
            WishAngVel::Force(force) => force.clamp(-1.0, 1.0) * self.angvel_max(),
        };

        let angacc_dt = self.angacc() * DT.as_secs_f32();
        let angvel_change = wish_angvel.clamp(-angacc_dt, angacc_dt);
        if angvel_change.abs() > 0.01 {
            self.angvel += angvel_change;
        }

        // TODO: Thruster
    }
}
