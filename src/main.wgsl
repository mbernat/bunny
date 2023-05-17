struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let id = f32(in_vertex_index);
    return vec4(0.5 * (id - 1.0), 0.5 * (1.0 - abs(id - 1.0)), 0.0, 1.0);
}

@fragment
fn fragment(@builtin(position) input: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(1.0);
}