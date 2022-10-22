use crate::constants::COLOR_ALICE_BLUE;
use gdnative::{api::VisualServer, prelude::*};

pub struct CanvasItemApi {
    vs: &'static VisualServer,
    pub parent: Rid,
    pub item: Rid,
    pub has_add_tr: bool,
    pub free_item_on_drop: bool,
}
impl CanvasItemApi {
    pub fn new(canvas_item: Rid, parent: Rid, free_item_on_drop: bool) -> Self {
        unsafe {
            Self {
                vs: VisualServer::godot_singleton(),
                parent,
                item: canvas_item,
                has_add_tr: false,
                free_item_on_drop,
            }
        }
    }

    /// Parent can be another item or a canvas.
    pub fn new_child_of(parent: Rid, free_item_on_drop: bool) -> Self {
        unsafe {
            let vs = VisualServer::godot_singleton();
            let item = vs.canvas_item_create();
            vs.canvas_item_set_parent(item, parent);
            Self {
                vs,
                parent,
                item,
                has_add_tr: false,
                free_item_on_drop,
            }
        }
    }

    /// Parent can be another item or a canvas.
    pub fn set_parent(&mut self, parent: Rid) {
        unsafe {
            self.vs.canvas_item_set_parent(self.item, parent);
            self.parent = parent;
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            self.vs.canvas_item_clear(self.item);
        }
        self.has_add_tr = false;
    }

    pub fn add_circle(&mut self, pos: Vector2, radius: f32, color: Option<Color>) {
        unsafe {
            self.clear_add_transform();

            self.vs.canvas_item_add_circle(
                self.item,
                pos,
                radius as f64,
                color.unwrap_or(COLOR_ALICE_BLUE),
            );
        }
    }

    pub fn add_cuboid(&mut self, pos: Vector2, rot: f32, size: Vector2, color: Option<Color>) {
        unsafe {
            let mut tr = Transform2D::IDENTITY;
            tr.set_rotation(rot);
            tr.origin = pos;
            self.add_transform(tr);

            self.vs.canvas_item_add_rect(
                self.item,
                Rect2 {
                    position: Vector2::ZERO,
                    size,
                },
                color.unwrap_or(COLOR_ALICE_BLUE),
            );
        }
    }

    pub fn add_polyline(&mut self, points: PoolArray<Vector2>, width: f32, color: Option<Color>) {
        unsafe {
            self.clear_add_transform();

            self.vs.canvas_item_add_polyline(
                self.item,
                points,
                PoolArray::from_iter(std::iter::once(color.unwrap_or(COLOR_ALICE_BLUE))),
                width as f64,
                true,
            )
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            self.vs.canvas_item_set_visible(self.item, visible);
        }
    }

    pub fn set_transform(&self, transform: Transform2D) {
        unsafe { self.vs.canvas_item_set_transform(self.item, transform) }
    }

    pub fn free(self) {
        unsafe {
            self.vs.free_rid(self.item);
        }
    }

    fn add_transform(&mut self, tr: Transform2D) {
        unsafe {
            self.vs.canvas_item_add_set_transform(self.item, tr);
            self.has_add_tr = true;
        }
    }

    fn clear_add_transform(&mut self) {
        unsafe {
            if self.has_add_tr {
                self.vs
                    .canvas_item_add_set_transform(self.item, Transform2D::IDENTITY);
                self.has_add_tr = false;
            }
        }
    }
}
impl Drop for CanvasItemApi {
    fn drop(&mut self) {
        if self.free_item_on_drop {
            unsafe {
                self.vs.free_rid(self.item);
            }
        }
    }
}
