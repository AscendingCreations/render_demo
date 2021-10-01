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
    [[builtin(vertex_index)]] my_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    out.zpos = vertex.position.z;
    out.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    out.tex_coords = vertex.tex_coords.xyz;
    return out;
}

[[group(1), binding(0)]]
var tex: texture_2d_array<f32>;
[[group(1), binding(1)]]
var sample: sampler;

[[group(2), binding(0)]]
var maptex: texture_2d<u32>;

// Fragment shader
[[stage(fragment)]]
fn main(in: VertexOutput,) -> [[location(0)]] vec4<f32> {
    let yoffset = abs((i32(in.zpos) - 8) * 32);
    let tile_pos = vec2<i32> (i32(floor(in.tex_coords.x / 16.0)), i32(floor(in.tex_coords.y / 16.0)) + yoffset);
    let tile: vec4<u32> = textureLoad(maptex, tile_pos.xy, 0);

    let pos = vec2<i32>((i32(tile.r) % 128) * 16 + (i32(in.tex_coords.x) % 16), i32(floor((f32(tile.r) / 128.0))) * 16 + (i32(in.tex_coords.y) % 16));
    let layer = i32(tile.g);

    if (layer > 0) {
        let layer = layer - i32(1);
    }

    let object_color: vec4<f32>  = textureLoad(tex, pos.xy, layer, 0);
    let alpha = object_color.a * (f32(tile.a) / 100.0);

    return vec4<f32>(object_color.rgb, alpha);
}