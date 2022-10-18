use crate::constants::COLOR_ALICE_BLUE;
use crate::util::*;
use battlescape::*;
use gdnative::api::*;
use gdnative::prelude::Node2D;
use gdnative::prelude::*;
use rapier2d::data::Arena;


// TODO: Use glam!
#[derive(Default)]
pub struct BattlescapeSnapshot {
    pub tick: u64,
    pub bound: f32,
    pub hulls: Arena<Hull>,
    pub ships: Arena<Ship>,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
}
impl BattlescapeSnapshot {
    pub fn take_snapshot(&mut self, bc: &Battlescape) {
        self.tick = bc.tick;
        self.bound = bc.bound;
        bc.hulls.clone_into(&mut self.hulls);
        bc.ships.clone_into(&mut self.ships);
        bc.physics.bodies.clone_into(&mut self.bodies);
        bc.physics.colliders.clone_into(&mut self.colliders);
    }

    pub fn draw_lerp(from: &Self, to: &Self, owner: &Node2D, weight: f32) {
        for (i, to_hull) in to.hulls.iter() {
            let to_body = to.bodies.get(to_hull.rb).unwrap();
            if let Some(from_hull) = from.hulls.get(i) {
                let from_body = from.bodies.get(from_hull.rb).unwrap();

                let body_pos = from_body.position().lerp_slerp(to_body.position(), weight);

                for &collider_handle in to_body.colliders().iter() {
                    let collider = to.colliders.get(collider_handle).unwrap();
                    match collider.shared_shape().shape_type() {
                        ShapeType::Ball => {
                            owner.draw_circle(
                                body_pos.translation.to_godot_scaled(),
                                body_pos.rotation.angle() as f64,
                                COLOR_ALICE_BLUE,
                            );
                        }
                        ShapeType::Cuboid => todo!(),
                        // ShapeType::Capsule => todo!(),
                        // ShapeType::Segment => todo!(),
                        // ShapeType::Triangle => todo!(),
                        // ShapeType::TriMesh => todo!(),
                        // ShapeType::Polyline => todo!(),
                        // ShapeType::HalfSpace => todo!(),
                        // ShapeType::HeightField => todo!(),
                        ShapeType::Compound => todo!(),
                        ShapeType::ConvexPolygon => todo!(),
                        // ShapeType::RoundCuboid => todo!(),
                        // ShapeType::RoundTriangle => todo!(),
                        // ShapeType::RoundConvexPolygon => todo!(),
                        ShapeType::Custom => todo!(),
                        _ => {}
                    }
                }
            } else {
                // This is new.
            }
        }
    }
}
