use super::*;
use godot::{engine::{RenderingServer, Texture2D}, prelude::*};

pub const ALICE_BLUE: Color = Color {
    r: 0.94,
    g: 0.97,
    b: 1.0,
    a: 1.0,
};

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

fn vs() -> &'static RenderingServer {
    &RenderingServer::singleton()
}

static mut CACHE: Option<Cache> = None;
fn cache() -> &'static mut Cache {
    unsafe {
        CACHE.get_or_insert_default()
    }
}
struct Cache {
    texture: AHashMap<String, Gd<Texture2D>>,
}
impl Default for Cache {
    fn default() -> Self {
        Self {
            texture: Default::default(),
        }
    }
}

pub fn texture(path: &str) -> &Gd<Texture2D> {
    let cache = cache();

    if let Some(tex) = cache.texture.get(path) {
        tex
    } else {
        let load = godot::engine::ResourceLoader::singleton();
        
        if let Some(tex) = load.load(path.into(), "Texture2D".into(), godot::engine::resource_loader::CacheMode::CACHE_MODE_REPLACE) {
            let tex = tex.cast::<Texture2D>();
            &cache.texture.entry(path.to_string()).or_insert(tex)
        } else {
            log::warn!("texture at {} not found", path);
            // TODO: Return debug texture.
            todo!()
        }
    }
}

pub struct DrawApi {
    pub item: RID,
    has_add_tr: bool,
}
impl DrawApi {
    fn new(parent: RID) -> Self {
        let vs = vs();
        let item = vs.canvas_item_create();
        vs.canvas_item_set_parent(item, parent);
        
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

    pub fn clear(&mut self) {
        vs().canvas_item_clear(self.item);
    }

    pub fn set_transform(&self, pos: Vector2, rot: f32) {
        // let mut tr = Transform2D::IDENTITY;
        // tr.origin = pos;
        // tr.set_rotation(rot);
        // vs().canvas_item_set_transform(self.item, tr);
    }

    pub fn add_circle(&mut self, pos: Vector2, radius: f32, color: Option<Color>) {
        self.clear_add_transform();

        vs().canvas_item_add_circle(
            self.item,
            pos,
            radius as f64,
            color.unwrap_or(ALICE_BLUE),
        );
    }

    pub fn add_cuboid(&mut self, pos: Vector2, rot: f32, size: Vector2, color: Option<Color>) {
        // let mut tr = Transform2D::IDENTITY;
        // tr.set_rotation(rot);
        // tr.origin = pos;
        // self.add_set_transform(tr);

        // vs().canvas_item_add_rect(
        //     self.item,
        //     Rect2 {
        //         position: Vector2::ZERO,
        //         size,
        //     },
        //     color.unwrap_or(ALICE_BLUE),
        // );
    }

    pub fn add_polyline(&mut self, points: Vector2Array, width: f32, color: Option<Color>) {
        // self.clear_add_transform();

        // vs().canvas_item_add_polyline(
        //     self.item,
        //     points,
        //     ColorArray::from_iter(std::iter::once(color.unwrap_or(ALICE_BLUE))),
        //     width as f64,
        //     true,
        // )
    }

    pub fn add_texture(&self, path: &str, centered: bool) {
        let tex = texture(path);

        let size = tex.get_size();

        // let position = if centered { size * -0.5 } else { Vector2::ZERO };
        
        // vs().canvas_item_add_texture_rect(
        //     self.item,
        //     Rect2 { position, size },
        //     tex.get_rid(),
        //     false,
        //     WHITE,
        //     false,
        // )
    }

    pub fn set_visible(&self, visible: bool) {
        vs().canvas_item_set_visible(self.item, visible);
    }

    pub fn free(&self) {
        vs().free_rid(self.item);
    }

    fn add_set_transform(&mut self, tr: Transform2D) {
        vs().canvas_item_add_set_transform(self.item, tr);
        self.has_add_tr = true;
    }

    fn clear_add_transform(&mut self) {
        // if self.has_add_tr {
        //     vs().canvas_item_add_set_transform(self.item, Transform2D::IDENTITY);
        //     self.has_add_tr = false;
        // } 
    }
}
impl Drop for DrawApi {
    fn drop(&mut self) {
        self.free();
    }
}
