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
    @location(2) tilesize: f32,
    @location(3) texture_id: f32,
    @location(4) texture_layer: f32,
    @location(5) color: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) uv_layer: i32,
    @location(2) color: vec4<f32>,
};

@group(1)
@binding(0)
var tex: texture_2d_array<f32>;
@group(1)
@binding(1)
var tex_sample: sampler;

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((color & 0xff0000u) >> 16u),
        f32((color & 0xff00u) >> 8u),
        f32((color & 0xffu)),
        f32((color & 0xff000000u) >> 24u),
    ) / 255.0;
}

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    var pos = vertex.position;
    let v = vertex.vertex_idx % 4u;
    let size = textureDimensions(tex);
    let total_tiles = size.x / u32(vertex.tilesize);
    let tileposx = f32(u32(vertex.texture_id) % total_tiles);
    let tileposy = f32(u32(vertex.texture_id) / total_tiles);

    switch v {
        case 1u: {
            result.uv = vec2<f32>(tileposx + vertex.tilesize, tileposy + vertex.tilesize);
            pos.x += vertex.tilesize;
        }
        case 2u: {
            result.uv = vec2<f32>(tileposx + vertex.tilesize, tileposy);
            pos.x += vertex.tilesize;
            pos.y += vertex.tilesize;
        }
        case 3u: {
            result.uv = vec2<f32>(tileposx, tileposy);
            pos.y += vertex.tilesize;
        }
        default: {
            result.uv = vec2<f32>(tileposx, tileposy + vertex.tilesize);
        }
    }

    result.clip_position =  (global.proj * global.view) * vec4<f32>(pos, 1.0);
    result.color = unpack_color(vertex.color);
    result.uv_layer = i32(vertex.texture_layer);
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let object_color = textureSampleLevel(tex, tex_sample, vertex.uv, vertex.uv_layer, 1.0);

    let color = object_color * vertex.color;

    if (color.a <= 0.0) {
        discard;
    }

    return color;
}

