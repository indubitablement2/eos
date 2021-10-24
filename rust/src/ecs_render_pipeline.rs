use crate::constants::*;
use gdnative::api::*;
use gdnative::prelude::*;
use std::convert::TryInto;

/// All that is needed to render sprites.
pub struct RenderRes {
    pub canvas_rid: Rid,
    pub mesh_rid: Rid,
    pub multimesh_rid: Rid,
    // pub texture: Ref<TextureArray>,
    pub texture_rid: Rid,
    /// Unused.
    pub normal_texture_rid: Rid,
    /// This is the bulk array that is needed for the multimesh.
    pub render_data: Vec<f32>,
    /// This is used to cull extra uneeded sprite from render_data.
    pub visible_instance: i64,
}
impl RenderRes {
    pub fn new(owner: &Node2D) -> Self {
        let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };

        // Create mesh.
        let mesh_rid = create_basic_mesh(visual_server);

        // Create multimesh.
        let multimesh_rid = visual_server.multimesh_create();
        visual_server.multimesh_set_mesh(multimesh_rid, mesh_rid);
        visual_server.multimesh_allocate(
            multimesh_rid,
            NUM_RENDER.into(),
            VisualServer::MULTIMESH_TRANSFORM_2D,
            VisualServer::MULTIMESH_COLOR_NONE,
            VisualServer::MULTIMESH_CUSTOM_DATA_8BIT,
        );

        Self {
            canvas_rid: owner.get_canvas(),
            mesh_rid,
            multimesh_rid,
            texture_rid: todo!(),
            normal_texture_rid: Rid::new(),
            render_data: Vec::with_capacity(BULK_ARRAY_SIZE),
            visible_instance: 0,
        }
    }
}

/// Make a basic 1 by 1 mesh. That goes from -0.5 to 0.5.
fn create_basic_mesh(visual_server: &VisualServer) -> Rid {
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

    let mesh_rid = visual_server.mesh_create();

    visual_server.mesh_add_surface_from_arrays(
        mesh_rid,
        VisualServer::PRIMITIVE_TRIANGLES,
        arr.into_shared(),
        VariantArray::new().into_shared(),
        97280,
    );

    mesh_rid
}
