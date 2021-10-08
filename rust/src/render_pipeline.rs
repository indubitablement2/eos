use gdnative::api::*;
use gdnative::prelude::*;
use std::convert::TryInto;
use std::mem::replace;
use crate::ecs_resources::RenderRes;

/// All that is needed to render sprites.
pub struct RenderPipeline {
    pub canvas_rid: Rid,
    pub mesh_rid: Rid,
    pub multimesh_rid: Rid,
    pub multimesh_allocate: i32,
    pub texture: Ref<TextureArray>,
    pub texture_rid: Rid,
    /// Unused.
    pub normal_texture_rid: Rid,
    /// RenderRes taken from the Ecs.
    pub maybe_render_res: Option<RenderRes>,
}

impl RenderPipeline {
    pub fn new(owner: &Node2D) -> Self {
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
            VisualServer::MULTIMESH_CUSTOM_DATA_FLOAT,
        );

        make_atlas();
        let texture = TextureArray::new();
        let texture_rid = texture.get_rid();
        // todo: material and shader params.

        // todo: Add ships textures.

        Self {
            canvas_rid,
            mesh_rid,
            multimesh_rid,
            multimesh_allocate,
            texture: texture.into_shared(),
            texture_rid,
            normal_texture_rid: Rid::new(),
            maybe_render_res: None,
        }
    }

    pub fn render(&mut self) {
        if let Some(render_res) = self.maybe_render_res.take() {
            let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };

            visual_server.multimesh_set_as_bulk_array(self.multimesh_rid, render_res.render_data);
            visual_server.multimesh_set_visible_instances(self.multimesh_rid, render_res.visible_instance);
            visual_server.canvas_item_add_multimesh(
                self.canvas_rid,
                self.multimesh_rid,
                self.texture_rid,
                self.normal_texture_rid,
            );
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
        indices_write[0] = 0;
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
        97280,
    );
}

fn make_atlas() -> () {
    let dir = Directory::new();

    // Check if atlas already exist.

    // Get all sprites name.
    let mut sprites_names = Vec::with_capacity(1024);
    if dir.open("res://sprites").is_err() {
        godot_error!("Could not open 'res://sprites'.");
        return;
    }
    if dir.list_dir_begin(true, true).is_err() {
        godot_error!("Could not list dir.");
        return;
    }
    let mut file_name = dir.get_next().to_string();
    while !file_name.is_empty() {
        godot_print!("{}", file_name);
        sprites_names.push(replace(&mut file_name, dir.get_next().to_string()));
    }

    // Check if hash match.
}
