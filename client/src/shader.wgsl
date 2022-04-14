struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

struct FragmentOutput {
    [[location(0)]] frag_color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] in_vertex_index: u32,
    [[builtin(instance_index)]] in_intance_index: u32,
    [[location(0)]] vertex_position: vec2<f32>,
    [[location(1)]] instance_position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let x = (vertex_position.x + instance_position.x) * 0.1;
    let y = (vertex_position.y + instance_position.y) * 0.1;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(
    in: VertexOutput,
) -> FragmentOutput {
    var out: FragmentOutput;
    out.frag_color = vec4<f32>(0.5, 0.5, 1.0, 1.0); 
    return out;
}