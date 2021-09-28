[[block]]
struct Camera {
    view_proj: mat4x4<f32>;
    eye: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    vertex: VertexInput,
    [[builtin(vertex_index)]] my_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    return out;
}

[[group(1), binding(0)]]
var tex: texture_2d_array<f32>;
[[group(1), binding(1)]]
var sample: sampler;

// Fragment shader
[[stage(fragment)]]
fn main(in: VertexOutput,) -> [[location(0)]] vec4<f32> {
    let layer: i32 = i32(in.tex_coords.z);
    let object_color = textureSample(tex, sample, in.tex_coords.xy, layer);
    let alpha = mix(1.0, object_color.a, in.color.a );
    return vec4<f32>(object_color.rgb, alpha);
}