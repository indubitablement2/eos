use crate::prelude::*;
use ahash::AHashMap;
use crossbeam::channel::{bounded, Receiver, Sender};
use macroquad::models::{draw_mesh, Mesh};
use macroquad::shapes::*;

pub struct Rendering {
    /// Positin the camera is centered at.
    pub target: Vec2,
    /// Zoom in >1 or out <1.
    pub zoom: f32,

    /// Each map is a new layer.
    /// - key: texture/imagr path
    /// - value: batched draw the same texture with different position and rotation
    shaded_draws: Vec<AHashMap<&'static str, Vec<(Vec2, f32)>>>,

    /// Holds albedo, normal, specular and glow.
    geometry_render_target: RenderTarget,
    /// A mesh that is 4 quads.
    geometry_mesh: Mesh,

    screen_size: UVec2,

    /// texture path -> texture.
    /// TODO: Don't let texture cache get too large.
    cached_textures: AHashMap<&'static str, Texture2D>,
    image_load_futures: Vec<(&'static str, Receiver<Image>)>,
}
impl Rendering {
    /// The highest valid layer.
    pub const LAYER_MAX: usize = 10;

    /// Draw a shaded texture (albedo, normal, specular and glow).
    ///
    /// Layer can be 0 to `Self::LAYER_MAX`.
    /// Layer are draw in order,
    /// but texure in the same layer are draw in any order to maximize batching opportunity.
    pub fn shaded_draw(
        &mut self,
        path: &'static str,
        position: Vec2,
        rotation: f32,
        mut layer: usize,
    ) {
        if layer > Self::LAYER_MAX {
            log::warn!(
                "Can not draw on layer {}. Max {}. Clamping...",
                layer,
                Self::LAYER_MAX
            );
            layer = Self::LAYER_MAX;
        }

        self.shaded_draws[layer]
            .entry(path)
            .or_default()
            .push((position, rotation));
    }

    /// Draw data that was queued previously.
    pub fn draw(&mut self, rt: &Runtime) {
        self.check_screen_size_change();
        self.handle_futures();
        self.geo_pass(rt);
        self.final_pass();
    }

    fn check_screen_size_change(&mut self) {
        let screen_size = uvec2(screen_width() as u32, screen_height() as u32);

        if self.screen_size != screen_size {
            self.screen_size = screen_size;
            self.geometry_render_target.delete();
            self.geometry_render_target = render_target(screen_size.x * 2, screen_size.y * 2);

            log::debug!("Screen resized to {}.", screen_size);
        }
    }

    /// Receive newly loaded image and upload them to the gpu.
    /// Keep the texture in cache.
    fn handle_futures(&mut self) {
        self.image_load_futures
            .drain_filter(|(path, r)| match r.try_recv() {
                Ok(image) => {
                    *self
                        .cached_textures
                        .get_mut(path)
                        .expect("there should be an empty texture") = Texture2D::from_image(&image);
                    true
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    self.cached_textures
                        .remove(path)
                        .expect("there should be an empty texture");
                    true
                }
                _ => false,
            });
    }

    /// Render albedo, normal, specular and glow on separate corner of a large (screen * 2) texture
    /// to simulate multiple render target.
    fn geo_pass(&mut self, rt: &Runtime) {
        // Setup camera.
        let mut camera = Camera2D::from_display_rect(Rect {
            x: self.target.x - screen_width() * 0.5,
            y: self.target.y - screen_height() * 0.5,
            w: screen_width() * 2.0,
            h: screen_height() * 2.0,
        });
        camera.render_target = Some(self.geometry_render_target);
        set_camera(&camera);
        clear_background(GRAY);

        // Debugs.
        let p = camera.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        let r = (get_time().rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI) as f32;
        self.shaded_draw("ansg.png", p, r, 0);
        draw_rectangle_lines(
            -screen_width() * 0.5,
            -screen_height() * 0.5,
            screen_width(),
            screen_height(),
            16.0,
            YELLOW,
        );

        // Draw shaded textures.
        for layer in self.shaded_draws.iter_mut() {
            for (path, draws) in layer.drain() {
                let texture = load_cached_texture(
                    &mut self.cached_textures,
                    &mut self.image_load_futures,
                    path,
                    rt,
                );
                self.geometry_mesh.texture = Some(texture);

                // Draw batched.
                let scale = vec2(texture.width(), texture.height()) * 0.5;
                for (pos, rot) in draws {
                    let a = Affine2::from_scale_angle_translation(scale, rot, pos);

                    // Set mesh position.
                    self.geometry_mesh.vertices[0].position =
                        a.transform_point2(vec2(-1.0, -1.0)).extend(0.0);
                    self.geometry_mesh.vertices[1].position =
                        a.transform_point2(vec2(1.0, -1.0)).extend(0.0);
                    self.geometry_mesh.vertices[2].position =
                        a.transform_point2(vec2(1.0, 1.0)).extend(0.0);
                    self.geometry_mesh.vertices[3].position =
                        a.transform_point2(vec2(-1.0, 1.0)).extend(0.0);

                    draw_mesh(&self.geometry_mesh);
                    // draw_texture(texture, x, y, color)
                }
            }
        }
    }

    /// Final render to screen.
    fn final_pass(&mut self) {
        // Make view range from -1..1
        let camera = Camera2D::from_display_rect(Rect {
            x: -1.0,
            y: -1.0,
            w: 2.0,
            h: 2.0,
        });
        set_camera(&camera);
        draw_texture_ex(
            self.geometry_render_target.texture,
            -1.0,
            -1.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(2.0, 2.0)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: true,
                pivot: None,
            },
        );
    }
}
impl Default for Rendering {
    fn default() -> Self {
        Self {
            target: Vec2::ZERO,
            zoom: 1.0,
            shaded_draws: vec![AHashMap::new(); Self::LAYER_MAX + 1],
            screen_size: UVec2::ZERO,
            geometry_render_target: render_target(0, 0),
            geometry_mesh: geo_mesh(),
            cached_textures: Default::default(),
            image_load_futures: Default::default(),
        }
    }
}

fn geo_mesh() -> Mesh {
    Mesh {
        vertices: vec![
            macroquad::models::Vertex {
                position: vec3(0.0, 0.0, 0.0),
                uv: vec2(0.0, 0.0),
                color: WHITE,
            },
            macroquad::models::Vertex {
                position: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.5, 0.0),
                color: WHITE,
            },
            macroquad::models::Vertex {
                position: vec3(1.0, 1.0, 0.0),
                uv: vec2(0.5, 0.5),
                color: WHITE,
            },
            macroquad::models::Vertex {
                position: vec3(0.0, 1.0, 0.0),
                uv: vec2(0.0, 0.5),
                color: WHITE,
            },
        ],
        #[rustfmt::skip]
        indices: vec![
            0, 1, 2, 0, 2, 3,
            0, 1, 2, 0, 2, 3,
            0, 1, 2, 0, 2, 3,
            0, 1, 2, 0, 2, 3,
        ],
        texture: None,
    }
}

/// Load a texture from cache or request it to be loaded from disk.
///
/// When loading from disk, will return an empty texture in the meantime.
fn load_cached_texture(
    cached_textures: &mut AHashMap<&'static str, Texture2D>,
    image_load_futures: &mut Vec<(&'static str, Receiver<Image>)>,
    path: &'static str,
    rt: &Runtime,
) -> Texture2D {
    *cached_textures.entry(path).or_insert_with(|| {
        log::debug!("Loading: '{}'", path);
        let (s, r) = bounded(1);
        rt.spawn(load_image_from_file(path, s));
        image_load_futures.push((path, r));
        Texture2D::empty()
    })
}

async fn load_image_from_file(path: &'static str, out: Sender<Image>) {
    match load_image(path).await {
        Ok(img) => out.send(img).unwrap(),
        Err(err) => log::error!("{:?} while loading texture at '{}'", err, path),
    }
}
