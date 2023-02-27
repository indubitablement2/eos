use super::*;
use crate::util::*;
use battlescape::bs_client::*;
use battlescape::entity::{WishAngVel, WishLinVel};
use godot::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInputs {
    // TODO: Add configs: is_face_cursor_toggle: bool,
    // TODO: Keep actions active until handled by `to_client_inputs`
    /// If wish linvel is relative to current angle.
    relative_linvel: bool,
    face_cursor: bool,
    /// Try to cancel both angvel and linvel if not applying any force.
    cancel_vel: bool,
    /// In game units.
    mouse_pos: glam::Vec2,
    wish_dir: glam::Vec2,
    strafe: f32,
}
impl PlayerInputs {
    pub fn handle_input(&mut self) {
        // TODO: when inputs are working.
    }

    fn update(&mut self, node: &Node2D) {
        self.mouse_pos = node.get_global_mouse_position().to_glam_descaled();

        let input = Input::singleton();

        self.wish_dir.x = input.get_action_strength("right".into(), false) as f32
            - input.get_action_strength("left".into(), false) as f32;
        self.wish_dir.y = input.get_action_strength("down".into(), false) as f32
            - input.get_action_strength("up".into(), false) as f32;
        self.strafe = input.get_action_strength("strafe_right".into(), false) as f32
            - input.get_action_strength("strafe_left".into(), false) as f32;

        // TODO: Remove when inputs are working.
        self.cancel_vel = input.is_action_pressed("cancel_vel".into(), false);
        self.face_cursor = input.is_action_pressed("face_cursor".into(), false);

        self.wish_dir.x = self.wish_dir.x.clamp(-1.0, 1.0);
        self.wish_dir.y = self.wish_dir.y.clamp(-1.0, 1.0);
        self.strafe = self.strafe.clamp(-1.0, 1.0);
    }

    pub fn to_client_inputs(&mut self, node: &Node2D) -> ClientInputs {
        self.update(node);

        if self.face_cursor {
            // Cursor controls.

            let wish_linvel = if self.wish_dir.aprox_zero() && self.strafe.aprox_zero() {
                if self.cancel_vel {
                    WishLinVel::Cancel
                } else {
                    WishLinVel::Keep
                }
            } else {
                if self.relative_linvel {
                    WishLinVel::Relative {
                        force: glam::vec2(
                            (self.wish_dir.x + self.strafe).clamp(-1.0, 1.0),
                            -self.wish_dir.y,
                        )
                        .clamp_length_max(1.0)
                        .to_na(),
                    }
                } else {
                    WishLinVel::Absolute {
                        force: glam::vec2(
                            (self.wish_dir.x + self.strafe).clamp(-1.0, 1.0),
                            self.wish_dir.y,
                        )
                        .clamp_length_max(1.0)
                        .to_na(),
                    }
                }
            };

            let wish_angvel = WishAngVel::Aim {
                position: self.mouse_pos.to_na(),
            };

            ClientInputs {
                wish_linvel,
                wish_angvel,
            }
        } else {
            // Tank controls.

            let wish_linvel = if self.wish_dir.y.aprox_zero() && self.strafe.aprox_zero() {
                if self.cancel_vel {
                    WishLinVel::Cancel
                } else {
                    WishLinVel::Keep
                }
            } else {
                WishLinVel::Relative {
                    force: na::Vector2::new(self.strafe, -self.wish_dir.y),
                }
            };

            let wish_angvel = if self.wish_dir.x.aprox_zero() {
                WishAngVel::Cancel
            } else {
                WishAngVel::Force {
                    force: self.wish_dir.x,
                }
            };

            ClientInputs {
                wish_linvel,
                wish_angvel,
            }
        }
    }
}
