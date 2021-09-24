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
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    return out;
}

[[group(1), binding(0)]]
var texture: texture_2d_array<f32>;
[[group(1), binding(1)]]
var sample: sampler;

// Fragment shader
[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let object_color = textureSample(texture, sample, in.tex_coords.xy, in.tex_coords.z);
    let result = in.color.rgb * object_color.rgb;
    return vec4<f32>(result, in.color.a);
}