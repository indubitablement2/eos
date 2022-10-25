use crate::prelude::*;
use battlescape::*;

#[derive(Default)]
struct HullSnapshot {
    hull_data_id: HullDataId,
    from: Affine2,
    to: Affine2,
}
impl HullSnapshot {
    pub fn new(hull_data_id: HullDataId, current_transform: Affine2) -> Self {
        Self {
            hull_data_id,
            from: current_transform,
            to: current_transform,
        }
    }

    fn update(&mut self, current_transform: Affine2) {
        self.from = self.to;
        self.to = current_transform;
    }
}

#[derive(Default)]
pub struct BattlescapeSnapshot {
    bound: f32,
    hulls: AHashMap<HullId, HullSnapshot>,
}
impl BattlescapeSnapshot {
    pub fn update_snapshot(&mut self, bc: &Battlescape) {
        self.bound = bc.bound;

        for (hull_id, hull) in bc.hulls.iter() {
            if let Some(rb) = bc.physics.bodies.get(hull.rb) {
                let current_transform = Affine2::from_mat3(Into::<Mat3>::into(rb.position().to_matrix()));

                self.hulls.entry(*hull_id).or_insert(HullSnapshot::new(hull.hull_data_id, current_transform)).update(current_transform);
                
            }
        }
    }

    pub fn draw_lerp(&self, weight: f32, rendering: &mut Rendering) {
        // let mat = load_material(
        //     "vertex_shader",
        //     "fragment_shader",
        //     MaterialParams {
        //         pipeline_params: PipelineParams {
        //             color_blend: Some(BlendState::new(
        //                 Equation::Add,
        //                 BlendFactor::Value(BlendValue::SourceAlpha),
        //                 BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        //             )),
        //             alpha_blend: None,
        //             ..Default::default()
        //         },
        //         uniforms: vec![],
        //         textures: vec![],
        //     },
        // )
        // .unwrap();

        // Camera2D
        // gl_use_material(material)
        // draw_texture(texture, x, y, color)

        // for (i, to_hull) in to.hulls.iter() {
        //     let to_body = to.bodies.get(to_hull.rb).unwrap();
        //     if let Some(from_hull) = from.hulls.get(i) {
        //         let from_body = from.bodies.get(from_hull.rb).unwrap();

        //         let body_pos = from_body.position().lerp_slerp(to_body.position(), weight);

        //         for &collider_handle in to_body.colliders().iter() {
        //             let collider = to.colliders.get(collider_handle).unwrap();
        //             match collider.shared_shape().as_typed_shape() {
        //                 TypedShape::Ball(ball) => {
        //                     owner.draw_circle(
        //                         body_pos.translation.to_godot_scaled(),
        //                         (ball.radius * GAME_TO_GODOT_RATIO) as f64,
        //                         COLOR_ALICE_BLUE,
        //                     );
        //                 }
        //                 TypedShape::Cuboid(cuboid) => {
        //                     owner.draw_set_transform(
        //                         Vector2::ZERO,
        //                         body_pos.rotation.angle() as f64,
        //                         Vector2::ZERO,
        //                     );
        //                     owner.draw_rect(
        //                         Rect2 {
        //                             position: body_pos.translation.to_godot_scaled(),
        //                             size: Vector2 {
        //                                 x: cuboid.half_extents.x,
        //                                 y: cuboid.half_extents.y,
        //                             },
        //                         },
        //                         COLOR_ALICE_BLUE,
        //                         true,
        //                         1.0,
        //                         false,
        //                     );
        //                 }
        //                 // TypedShape::Capsule(_) => todo!(),
        //                 // TypedShape::Segment(_) => todo!(),
        //                 // TypedShape::Triangle(_) => todo!(),
        //                 // TypedShape::TriMesh(_) => todo!(),
        //                 // TypedShape::Polyline(_) => todo!(),
        //                 // TypedShape::HalfSpace(_) => todo!(),
        //                 // TypedShape::HeightField(_) => todo!(),
        //                 TypedShape::Compound(_) => todo!(),
        //                 TypedShape::ConvexPolygon(poly) => {
        //                     // poly.points()
        //                 }
        //                 // TypedShape::RoundCuboid(_) => todo!(),
        //                 // TypedShape::RoundTriangle(_) => todo!(),
        //                 // TypedShape::RoundConvexPolygon(_) => todo!(),
        //                 TypedShape::Custom(_) => todo!(),
        //                 _ => {}
        //             }
        //         }
        //     } else {
        //         // This is new.
        //     }
        // }
    }
}