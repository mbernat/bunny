struct VertexInput {
    @location(0) position: vec3<f32>
}

struct VertexOutput {
    // 3d space? Values between -1 and 1
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
}

struct FragmentInput {
    // Screen space. Values between (0, 0) and (width, height)
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
}

@vertex
fn vertex(@builtin(vertex_index) index: u32, in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(in.position * 20.0, 1.0) + vec4(0.5, -2.0, 0.0, 1.0);
    let id = f32(index) / 1000.0;
    let r = i32(index) % 3 - 1;
    let g = 1 - i32(index) % 3;
    let b = 1 - abs(i32(index) % 3 - 1);
    out.color = id * vec4(f32(r), f32(g), f32(b), 1.0);
    return out;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return in.color;
}
