use super::*;
use godot::engine::Texture2D;

pub struct DrawCollider {
    pub collider_type: DrawColliderType,
    pub color: Color,
    pub position: Vector2,
    pub rotation: f32,
}

pub enum DrawColliderType {
    Circle { radius: f32 },
    Cuboid { half_size: Vector2 },
    Polygon { points: Vec<Vector2> },
    CompoundPolygon { polygons: Vec<Vec<Vector2>> },
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct DrawColliders {
    colliders: Vec<DrawCollider>,
    pixel: Gd<Texture2D>,
    base: Base<Node2D>,
}
impl DrawColliders {
    pub fn new_child(parent: &Gd<Node2D>) -> Gd<Self> {
        Gd::<Self>::with_base(|mut base| {
            add_child(parent, &base);

            base.hide();
            base.set_as_top_level(true);

            let obj = base.share();
            base.connect(
                "draw".into(),
                Callable::from_object_method(obj, "__draw"),
                0,
            );

            Self {
                colliders: Vec::new(),
                pixel: load("res://textures/pixel.png"),
                base,
            }
        })
    }

    pub fn enable_drawing(&mut self, colliders: Vec<DrawCollider>) {
        self.colliders = colliders;
        self.base.show();
        self.base.queue_redraw();
    }

    pub fn disable_drawing(&mut self) {
        self.base.hide();
        self.colliders = Default::default();
    }
}
#[godot_api]
impl DrawColliders {
    // TODO: Remove when _draw() is implemented.
    #[func]
    fn __draw(&mut self) {
        for shape in std::mem::take(&mut self.colliders).into_iter() {
            self.base
                .draw_set_transform(shape.position, shape.rotation as f64, Vector2::ONE);

            match shape.collider_type {
                DrawColliderType::Circle { radius } => {
                    self.base
                        .draw_circle(Vector2::ZERO, radius as f64, shape.color);
                }
                DrawColliderType::Cuboid { half_size } => {
                    // TODO: Need Rect2
                }
                DrawColliderType::Polygon { points } => {
                    let mut uvs = PackedVector2Array::new();
                    uvs.resize(points.len());
                    self.base.draw_colored_polygon(
                        PackedVector2Array::from(points.as_slice()),
                        shape.color,
                        uvs,
                        self.pixel.share(),
                    );
                }
                DrawColliderType::CompoundPolygon { polygons } => {
                    for points in polygons {
                        let mut uvs = PackedVector2Array::new();
                        uvs.resize(points.len());
                        self.base.draw_colored_polygon(
                            PackedVector2Array::from(points.as_slice()),
                            shape.color,
                            uvs,
                            self.pixel.share(),
                        );
                    }
                }
            }
        }
    }
}
