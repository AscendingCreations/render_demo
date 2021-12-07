[[block]]
struct Camera {
    view_proj: mat4x4<f32>;
    eye: vec3<f32>;
};

[[block]]
struct Time {
    seconds: f32;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[group(1), binding(0)]]
var<uniform> time: Time;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<u32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(1)]] col: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    out.col = vec4<f32>(f32(vertex.color.r) / 255.0, f32(vertex.color.g) / 255.0, f32(vertex.color.b) / 255.0, f32(vertex.color.a) / 255.0);

    return out;
}

// Fragment shader
[[stage(fragment)]]
fn fragment(in: VertexOutput,) -> [[location(0)]] vec4<f32> {
    if (in.col.a <= 0.0) {
        discard;
    }

    return in.col;
}