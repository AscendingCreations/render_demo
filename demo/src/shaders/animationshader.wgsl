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
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] tex_data: vec4<u32>;
    [[location(2)]] position: vec3<f32>;
    [[location(3)]] hue_alpha: vec2<u32>;
    [[location(4)]] frames: vec2<u32>;
    [[location(5)]] layer: i32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] tex_data: vec4<u32>;
    [[location(2)]] hue_alpha: vec2<u32>;
    [[location(3)]] frames: vec2<u32>;
    [[location(5)]] layer: i32;
};

[[stage(vertex)]]
fn main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.tex_data = vertex.tex_data;
    out.hue_alpha = vertex.hue_alpha;
    out.frames = vertex.frames;
    out.layer = vertex.layer;

    return out;
}

[[group(2), binding(0)]]
var tex: texture_2d_array<f32>;

fn hueShift(color: vec3<f32>, hue: f32) -> vec3<f32>
{
    var pi = 3.14159;
    let rad = hue * (pi/180.0);
    let k = vec3<f32>(0.57735, 0.57735, 0.57735);
    let cosAngle = cos(rad);
    return vec3<f32>(color * cosAngle + cross(k, color) * sin(rad) + k * dot(k, color) * (1.0 - cosAngle));
}

// Fragment shader
[[stage(fragment)]]
fn main(in: VertexOutput,) -> [[location(0)]] vec4<f32> {
    let id = time.seconds / (f32(in.frames[1]) / 1000.0);
    let frame = u32(floor(id % f32(in.frames[0])));
    let pos = vec2<i32>(i32((frame * in.tex_data[2]) + in.tex_data[0] + u32(in.tex_coords.x)), i32(in.tex_data[1] + u32(in.tex_coords.y)));
    let object_color = textureLoad(tex, pos.xy, in.layer, 0);
    let alpha = object_color.a * (f32(in.hue_alpha[1]) / 100.0);
    let color = hueShift(object_color.rgb, f32(in.hue_alpha[0]) % 361.0);
    return vec4<f32>(color, alpha);
}

