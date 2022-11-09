use crate::constants::{COLOR_ALICE_BLUE, COLOR_WHITE};
use ahash::AHashMap;
use gdnative::{api::VisualServer, prelude::*};

fn vs() -> &'static VisualServer {
    unsafe { VisualServer::godot_singleton() }
}

static mut CACHE: Option<Cache> = None;
fn cache() -> &'static mut Cache {
    unsafe {
        if CACHE.is_none() {
            CACHE = Some(Default::default());
        }

        if let Some(cache) = &mut CACHE {
            cache
        } else {
            unreachable!()
        }
    }
}
struct Cache {
    texture: AHashMap<&'static str, Ref<Texture>>,
}
impl Default for Cache {
    fn default() -> Self {
        Self {
            texture: Default::default(),
        }
    }
}

pub fn texture(path: &'static str) -> Ref<Texture> {
    let cache = cache();

    if let Some(tex) = cache.texture.get(path) {
        tex.clone()
    } else {
        let load = ResourceLoader::godot_singleton();
        if let Some(tex) = load.load(path, "Texture", false) {
            let tex = tex.cast::<Texture>().unwrap();
            cache.texture.insert(path, tex.clone());
            tex
        } else {
            log::error!("texture at {} not found", path);
            panic!();
        }
    }
}

pub struct DrawApi {
    pub item: Rid,
    has_add_tr: bool,
}
impl DrawApi {
    fn new(parent: Rid) -> Self {
        let vs = vs();
        let item = vs.canvas_item_create();
        unsafe {
            vs.canvas_item_set_parent(item, parent);
        }

        Self {
            item,
            has_add_tr: false,
        }
    }

    pub fn new_root(base: &Node2D) -> Self {
        Self::new(base.get_canvas())
    }

    pub fn new_child(&self) -> Self {
        Self::new(self.item)
    }

    pub fn visible(&self, visible: bool) {
        unsafe {
            vs().canvas_item_set_visible(self.item, visible);
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            vs().canvas_item_clear(self.item);
        }
    }

    pub fn set_transform(&self, pos: Vector2, rot: f32) {
        let mut tr = Transform2D::IDENTITY;
        tr.origin = pos;
        tr.set_rotation(rot);

        unsafe {
            vs().canvas_item_set_transform(self.item, tr);
        }
    }

    pub fn add_circle(&mut self, pos: Vector2, radius: f32, color: Option<Color>) {
        unsafe {
            self.clear_add_transform();

            vs().canvas_item_add_circle(
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
            self.add_set_transform(tr);

            vs().canvas_item_add_rect(
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

            vs().canvas_item_add_polyline(
                self.item,
                points,
                PoolArray::from_iter(std::iter::once(color.unwrap_or(COLOR_ALICE_BLUE))),
                width as f64,
                true,
            )
        }
    }

    pub fn add_texture(
        &mut self,
        albedo: &'static str,
        normal: Option<&'static str>,
        centered: bool,
    ) {
        unsafe {
            let tex = texture(albedo);
            let tex = tex.assume_safe();

            let size = tex.get_size();

            let normal_map = if let Some(path) = normal {
                let tex = texture(path);
                tex.assume_safe().get_rid()
            } else {
                Rid::new()
            };

            let position = if centered {
                size * -0.5
            } else {
                Vector2::ZERO
            };

            vs().canvas_item_add_texture_rect(
                self.item,
                Rect2 {
                    position,
                    size,
                },
                tex.get_rid(),
                false,
                COLOR_WHITE,
                false,
                normal_map,
            )
        }
    }

    // pub fn add_tex

    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            vs().canvas_item_set_visible(self.item, visible);
        }
    }

    pub fn free(&self) {
        unsafe {
            vs().free_rid(self.item);
        }
    }

    fn add_set_transform(&mut self, tr: Transform2D) {
        unsafe {
            vs().canvas_item_add_set_transform(self.item, tr);
            self.has_add_tr = true;
        }
    }

    fn clear_add_transform(&mut self) {
        unsafe {
            if self.has_add_tr {
                vs().canvas_item_add_set_transform(self.item, Transform2D::IDENTITY);
                self.has_add_tr = false;
            }
        }
    }
}
impl Drop for DrawApi {
    fn drop(&mut self) {
        self.free();
    }
}
