struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>
}

struct VertexOutput {
    // 3d space? Values between -1 and 1
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
}

struct FragmentInput {
    // Screen space. Values between (0, 0) and (width, height)
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>
}

fn color_by_index(index: u32) -> vec4<f32> {
    let id = f32(index) / 1000.0;
    let r = i32(index) % 3 - 1;
    let g = 1 - i32(index) % 3;
    let b = 1 - abs(i32(index) % 3 - 1);
    return vec4(f32(r), f32(g), f32(b), 1.0);
}

@vertex
fn vertex(@builtin(vertex_index) index: u32, in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let big = 10.0;
    let phi = 0.5;
    let c = cos(phi);
    let s = sin(phi);
    let scale = mat4x4(
        big, 0.0, 0.0, 0.0,
        0.0, big, 0.0, 0.0,
        0.0, 0.0, big, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    let model = mat4x4(
        1.0, 0.0, 0.0, 0.0,
        0.0, c, -s, 0.0,
        0.0, s, c, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    let view = mat4x4(
        1.0, 0.0, 0.0, 0.2,
        0.0, 1.0, 0.0, -0.9,
        0.0, 0.0, 1.0, -0.2,
        0.0, 0.0, 0.0, 1.0,
    );

    out.position = vec4(in.position, 1.0) * scale * model * view;
    let normal = vec4(in.normal, 0.0) * model;
    out.normal = normal.xyz;
    //out.color = color_by_index(index);
    out.color = vec4(out.normal, 1.0);

    return out;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let light = vec3(1.0, 4.0, 1.0);
    let light_normal = -normalize(light);
    let intensity = clamp((1.0 + dot(in.normal, light_normal)) / 2.0, 0.0, 1.0);
    return in.color * vec4(vec3(0.8 * intensity + 0.2), 1.0);

    //return in.color;
}
