struct Camera {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    eye: vec3<f32>,
    scale: f32,
};

struct Time {
    seconds: f32,
};

struct Screen {
    size: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<uniform> time: Time;

@group(0)
@binding(2)
var<uniform> screen: Screen;

struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) v_pos: vec2<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) hw: vec2<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) layer: u32,
    @location(5) color: u32,
    @location(6) is_color: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) layer: i32,
    @location(5) is_color: u32,
};

@group(1)
@binding(0)
var tex: texture_2d_array<f32>;
@group(1)
@binding(1)
var tex_sample: sampler;

@group(2)
@binding(0)
var emoji_tex: texture_2d_array<f32>;
@group(2)
@binding(1)
var emoji_tex_sample: sampler;

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
    var pos = vertex.pos;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));
    let v = vertex.vertex_idx % 4u;

    switch v {
        case 1u: {
            result.uv = vec2<f32>(vertex.uv.x + vertex.hw.x, vertex.uv.y + vertex.hw.y) /  fsize;
            pos.x += vertex.hw.x;
        }
        case 2u: {
            result.uv = vec2<f32>(vertex.uv.x + vertex.hw.x, vertex.uv.y) /  fsize;
            pos.x += vertex.hw.x;
            pos.y += vertex.hw.y;
        }
        case 3u: {
            result.uv = vec2<f32>(vertex.uv.x, vertex.uv.y) /  fsize;
            pos.y += vertex.hw.y;
        }
        default: {
            result.uv = vec2<f32>(vertex.uv.x, vertex.uv.y + vertex.hw.y) /  fsize;
        }
    }

    result.position = camera.proj * vec4<f32>(pos.xyz, 1.0);
    result.size = fsize;
    result.color = unpack_color(vertex.color);
    result.layer = i32(vertex.layer);
    result.v_pos = vertex.v_pos;
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let object_color = textureSample(tex, tex_sample, vertex.uv.xy, vertex.layer);

    if object_color.r <= 0.0 {
        discard;
    }

    return vertex.color.rgba * object_color.r;
}