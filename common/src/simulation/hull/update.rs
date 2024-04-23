use super::*;

impl Hull {
    pub fn update(
        &mut self,
        current: HullId,
        physics: &Physics,
        hulls: &mut Arena<HullId, Hull>,
    ) -> Option<RemoveReason> {
        // TODO: ai, etc

        self._apply_wish_angvel();
        self._apply_wish_linvel();

        None
    }

    fn _apply_wish_linvel(&mut self) {
        let wish_linvel = match self.wish_linvel {
            WishLinVel::None => return,
            WishLinVel::Keep => self.linvel.cap_magnitude(self.max_linear_velocity),
            WishLinVel::Cancel => Vector2::zeros(),
            WishLinVel::PositionSmooth(wish_pos) => {
                let to_pos = wish_pos - self.pos.translation.vector;
                if to_pos.magnitude_squared() < 0.5 {
                    vector![0.0, 0.0]
                } else {
                    to_pos.cap_magnitude(self.max_linear_velocity)
                }
            }
            WishLinVel::PositionOvershoot(wish_pos) => {
                let to_pos = wish_pos - self.pos.translation.vector;
                to_pos.try_normalize(0.1).unwrap_or(vector![0.0, 1.0]) * self.max_linear_velocity
            }
            WishLinVel::ForceAbsolute(force) => force.cap_magnitude(1.0) * self.max_linear_velocity,
            WishLinVel::ForceRelative(force) => {
                self.pos
                    .rotation
                    .transform_vector(&force.cap_magnitude(1.0))
                    * self.max_linear_velocity
            }
        };

        let linvel_change =
            (wish_linvel - self.linvel).cap_magnitude(self.linear_acceleration * DT.as_secs_f32());
        if linvel_change.x.abs() > 0.001 || linvel_change.y.abs() > 0.001 {
            self.linvel += linvel_change;
        }

        // TODO: Thruster
        // thruster_linear = wish_linvel;
    }

    fn _apply_wish_angvel(&mut self) {
        let wish_angvel: f32 = match self.wish_angvel {
            WishAngVel::None => return,
            WishAngVel::Keep => todo!(),
            WishAngVel::Stop => 0.0,
            WishAngVel::AimSmooth(_) => todo!(),
            WishAngVel::Force(_) => todo!(),
        };

        let angacc_dt = self.angular_acceleration * DT.as_secs_f32();
        let angvel_change = wish_angvel.clamp(-angacc_dt, angacc_dt);
        if angvel_change.abs() > 0.01 {
            self.angvel += angvel_change;
        }
        // TODO: Thruster
        // thruster_angular = wish_angvel;
    }
}
