use gdnative::api::*;
use gdnative::prelude::*;
use std::convert::TryInto;

/// Make a basic 1 by 1 mesh.
pub fn init_basic_mesh(visual_server: &VisualServer, mesh_rid: Rid) {
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

/// Return the number the string ends with.
/// # Examples
/// "hello_123_0555" would return Some("0555").
pub fn get_number_sufix(s: &str) -> Option<String> {
    for a in s[..0] {
        
    }
}
#[test]
fn test_get_number_sufix() {

}