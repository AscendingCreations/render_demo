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
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec3<f32>;
    [[location(1)]] zpos: f32;
};

[[stage(vertex)]]
fn main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.zpos = vertex.position.z;
    out.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    out.tex_coords = vertex.tex_coords.xyz;
    return out;
}

[[group(1), binding(0)]]
var tex: texture_2d_array<f32>;

[[group(2), binding(0)]]
var maptex: texture_2d<u32>;

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
    let yoffset = abs((i32(in.zpos) - 8) * 32);
    let coords = vec3<i32> (i32(in.tex_coords.x), i32(in.tex_coords.y), i32(in.tex_coords.z));
    let tile_pos = vec2<i32>(coords.x / 16, (coords.y / 16) + yoffset);
    let tile = textureLoad(maptex, tile_pos.xy, 0);
    let pos = vec2<i32>(i32(tile.r % 128u) * 16 + (coords.x % 16), i32(tile.r / 128u) * 16 + (coords.y % 16));
    let object_color = textureLoad(tex, pos.xy, i32(tile.g), 0);
    let alpha = object_color.a * (f32(tile.a) / 100.0);
    let color = hueShift(object_color.rgb, f32(tile.b) % 361.0);
    return vec4<f32>(color, alpha);
}

