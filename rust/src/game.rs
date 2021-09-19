use std::convert::TryInto;

use crate::battlescape::*;
use ahash::AHashMap;
use gdnative::api::*;
use gdnative::prelude::*;

/// Layer between godot and rust.
/// Godot is used for input/rendering. Rust is used for game logic.
#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_builder)]
pub struct Game {
    battlescapes: AHashMap<u64, Battlescape>,
    render_res: RenderRes,
}

/// All that is needed to render sprites.
struct RenderRes {
    pub canvas_rid: Rid,
    pub mesh_rid: Rid,
    pub multimesh_rid: Rid,
    pub multimesh_allocate: i32,
    pub texture_rid: Rid,
    pub normal_texture_rid: Rid,
    pub render_data: Option<(TypedArray<f32>, i64)>,
}

#[methods]
impl Game {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(owner: &Node2D) -> Self {
        Game {
            battlescapes: AHashMap::with_capacity(4),
            render_res: RenderRes::new(owner),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node2D) {

    }

    /// Free the rid we created.
    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node2D) {
        let visual_server = gdnative::api::VisualServer::godot_singleton();
        visual_server.free_rid(self.render_res.multimesh_rid);
        visual_server.free_rid(self.render_res.mesh_rid);
    }

    #[export]
    unsafe fn _process(&mut self, _owner: &Node2D, _delta: f32) {
        // start battlescape
        let num_render = self.render_res.multimesh_allocate;
        self.battlescapes.values_mut().for_each(|bc| {
            let update_request = UpdateRequest {
                send_render: Some(num_render),
                spawn_ship: Option::None,
            };
            if bc.update(update_request).is_err() {
                // TODO: Remove battlescape.
                godot_error!("Can not send to Battlescape. It probably crashed.");
            }
        });

        // todo: wait for result
    }

    #[export]
    unsafe fn _draw(&mut self, _owner: &Node2D, _enable: bool) {
        if let Some((render_data, num_instances)) = self.render_res.render_data.take() {
            let visual_server = gdnative::api::VisualServer::godot_singleton();
            visual_server.multimesh_set_as_bulk_array(self.render_res.multimesh_rid, render_data);
            visual_server.multimesh_set_visible_instances(self.render_res.multimesh_rid, num_instances);
            visual_server.canvas_item_add_multimesh(
                self.render_res.canvas_rid,
                self.render_res.multimesh_rid,
                self.render_res.texture_rid,
                self.render_res.normal_texture_rid,
            );
        }
    }

    #[export]
    unsafe fn spawn_ship(&mut self, _owner: &Node2D, position: Vector2, rotation: f32) {}
}

impl RenderRes {
    fn new(owner: &Node2D) -> RenderRes {
        let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };

        let canvas_rid = owner.get_canvas_item();

        let mesh_rid = visual_server.mesh_create();
        init_basic_mesh(visual_server, mesh_rid);

        // todo: Load that from setting.
        let multimesh_allocate = 20000i32;

        let multimesh_rid = visual_server.multimesh_create();
        visual_server.multimesh_set_mesh(multimesh_rid, mesh_rid);
        visual_server.multimesh_allocate(
            multimesh_rid, 
            multimesh_allocate.into(),
            VisualServer::MULTIMESH_TRANSFORM_2D,
            VisualServer::MULTIMESH_COLOR_NONE,
            VisualServer::MULTIMESH_CUSTOM_DATA_FLOAT
        );
        
        let texture = ResourceLoader::godot_singleton().load("path", "", true);

        // todo: material and shader params.
        
        // todo: Add ships textures.

        RenderRes {
            canvas_rid,
            mesh_rid,
            multimesh_rid,
            multimesh_allocate,
            texture_rid: Rid::new(),
            normal_texture_rid: Rid::new(),
            render_data: Option::None,
        }
    }
}

/// Make a basic 1 by 1 mesh.
fn init_basic_mesh(visual_server: &VisualServer, mesh_rid: Rid) {
    let mut vertices: TypedArray<Vector2> = TypedArray::new();
    vertices.resize(4);
    {
        let mut vertices_write = vertices.write();
        vertices_write[0] = Vector2::new(-0.5, -0.5);
        vertices_write[1] = Vector2::new(0.5, -0.5);
        vertices_write[2] = Vector2::new(0.5, 0.5);
        vertices_write[3] = Vector2::new(-0.5, 0.5);
    }

    let mut uvs: TypedArray<Vector2> = TypedArray::new();
    uvs.resize(4);
    {
        let mut uvs_write = uvs.write();
        uvs_write[0] = Vector2::new(0.0, 0.0);
        uvs_write[1] = Vector2::new(1.0, 0.0);
        uvs_write[2] = Vector2::new(1.0, 1.0);
        uvs_write[3] = Vector2::new(0.0, 1.0);
    }

    let mut colors: TypedArray<Color> = TypedArray::new();
    colors.resize(4);
    {
        let mut colors_write = colors.write();
        colors_write.fill(Color::from_rgba(1.0, 1.0, 1.0, 1.0));
    }

    let mut indices: TypedArray<i32> = TypedArray::new();
    indices.resize(6);
    {
        let mut indices_write = indices.write();
        indices_write[0] = 1;
        indices_write[1] = 1;
        indices_write[2] = 2;
        indices_write[3] = 2;
        indices_write[4] = 3;
        indices_write[5] = 0;
    }

    let arr = VariantArray::new();
    arr.resize(VisualServer::ARRAY_MAX.try_into().unwrap());
    arr.set(VisualServer::ARRAY_VERTEX.try_into().unwrap(), vertices);
    arr.set(VisualServer::ARRAY_TEX_UV.try_into().unwrap(), uvs);
    arr.set(VisualServer::ARRAY_COLOR.try_into().unwrap(), colors);
    arr.set(VisualServer::ARRAY_INDEX.try_into().unwrap(), indices);

    visual_server.mesh_add_surface_from_arrays(
        mesh_rid, 
        VisualServer::PRIMITIVE_TRIANGLES, 
        arr.into_shared(),
        VariantArray::new().into_shared(),
        97280
    );
}