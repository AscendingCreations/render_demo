struct Global {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    eye: vec3<f32>,
    scale: f32,
    size: vec2<f32>,
    seconds: f32,
};

@group(0)
@binding(0)
var<uniform> global: Global;

struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) v_pos: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) hw: vec2<f32>,
    @location(3) layer: i32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) zpos: f32,
    @location(2) layer: i32,
};

@group(1)
@binding(0)
var tex: texture_2d_array<f32>;
@group(1)
@binding(1)
var tex_sample: sampler;

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    var pos = vertex.position;
    let v = vertex.vertex_idx % 4u;

    switch v {
        case 1u: {
            result.tex_coords = vec2<f32>(vertex.hw.x, vertex.hw.y);
            pos.x += vertex.hw.x;
        }
        case 2u: {
            result.tex_coords = vec2<f32>(vertex.hw.x, 0.0);
            pos.x += vertex.hw.x;
            pos.y += vertex.hw.y;
        }
        case 3u: {
            result.tex_coords = vec2<f32>(0.0, 0.0);
            pos.y += vertex.hw.y;
        }
        default: {
            result.tex_coords = vec2<f32>(0.0, vertex.hw.y);
        }
    }

    result.zpos = pos.z;
    result.clip_position =  (global.proj * global.view) * vec4<f32>(pos, 1.0);
    result.layer = vertex.layer;
    return result;
}

@group(2)
@binding(0)
var maptex: texture_2d_array<u32>;

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let yoffset = abs((i32(vertex.zpos) - 8) * 32);
    let coords = vec3<i32> (i32(vertex.tex_coords.x + .5), i32(vertex.tex_coords.y + .5), vertex.layer);
    let tile_pos = vec2<i32>(coords.x / 16, (coords.y / 16) + yoffset);
    let tile = textureLoad(maptex, tile_pos.xy, coords.z, 0);
    let pos = vec2<i32>(i32(tile.r % 128u) * 16 + (coords.x % 16), i32(tile.r / 128u) * 16 + (coords.y % 16));
    let object_color = textureLoad(tex, pos.xy, i32(tile.g), 0);
    let alpha = object_color.a * (f32(tile.a) / 255.0);

    if (alpha <= 0.0) {
        discard;
    }

    return vec4<f32>(object_color.rgb, alpha);
}

