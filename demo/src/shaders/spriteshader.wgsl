struct Camera {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    eye: vec3<f32>,
};

struct Time {
    seconds: f32,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<uniform> time: Time;

struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) position: vec3<f32>,
    @location(1) tex_data: vec2<u32>,
    @location(2) color: u32,
    @location(3) frames: u32,
    @location(4) animate: u32,
    @location(5) time: u32,
    @location(6) layer: i32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_data: vec4<u32>,
    @location(2) col: vec4<f32>,
    @location(3) frames: vec2<u32>,
    @location(4) size: vec2<f32>,
    @location(5) layer: i32,
    @location(6) time: u32,
    @location(7) animate: u32,
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
    let v = vertex.vertex_idx % 4u;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));
    let tex_data = vec4<u32>(
        u32(vertex.tex_data[0] & 0xffffu), 
        u32((vertex.tex_data[0] & 0xffff0000u) >> 16u),
        u32(vertex.tex_data[1] & 0xffffu),
        u32((vertex.tex_data[1] & 0xffff0000u) >> 16u) 
    );

    result.clip_position = (camera.proj * camera.view) * vec4<f32>(vertex.position.xyz, 1.0);

    switch v {
        case 1u: {
            result.tex_coords = vec2<f32>(f32(tex_data[2]), f32(tex_data[3]));
        }
        case 2u: {
            result.tex_coords = vec2<f32>(f32(tex_data[2]), 0.0);
        }
        case 3u: {
            result.tex_coords = vec2<f32>(0.0, 0.0);
        }
        default: {
            result.tex_coords = vec2<f32>(0.0, f32(tex_data[3]));
        }
    }

    result.tex_data = tex_data;
    result.layer = vertex.layer;
    result.col = unpack_color(vertex.color);
    result.frames = vec2<u32>(u32(vertex.frames & 0xffffu), u32((vertex.frames & 0xffff0000u) >> 16u));
    result.size = fsize;
    result.animate = vertex.animate;
    result.time = vertex.time;
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var coords = vec2<f32>(0.0, 0.0);
    let xframes = vertex.frames[0];
    var yframes = vertex.frames[0];

    if (vertex.animate > 0u) {
        let id = time.seconds / (f32(vertex.time) / 1000.0);
        let frame = u32(floor(id % f32(xframes)));

        if (vertex.frames[1] > 0u) {
            yframes = vertex.frames[1];
        }

        coords = vec2<f32>(
            (f32(((frame % yframes) * vertex.tex_data[2]) + vertex.tex_data[0]) + vertex.tex_coords.x) / vertex.size.x,
            (f32(((frame / yframes) * vertex.tex_data[3]) + vertex.tex_data[1]) + vertex.tex_coords.y) / vertex.size.y
        );
    } else {
        coords = vec2<f32>(
            (f32(vertex.tex_data[0]) + vertex.tex_coords.x) / vertex.size.x,
            (f32(vertex.tex_data[1]) + vertex.tex_coords.y) / vertex.size.y
        );
    }

    var step = vec2<f32>(0.5, 0.5);
    var tex_pixel = vertex.size * coords - step.xy / 2.0;

    let corner = floor(tex_pixel) + 1.0;
    let frac = min((corner - tex_pixel) * vec2<f32>(2.0, 2.0), vec2<f32>(1.0, 1.0));

    var c1 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, 0.0)) + 0.5) / vertex.size, vertex.layer);
    var c2 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(step.x, 0.0)) + 0.5) / vertex.size, vertex.layer);
    var c3 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, step.y)) + 0.5) / vertex.size, vertex.layer);
    var c4 = textureSample(tex, tex_sample, (floor(tex_pixel + step.xy) + 0.5) / vertex.size, vertex.layer);

    c1 = c1 * (frac.x * frac.y);
    c2 = c2 *((1.0 - frac.x) * frac.y);
    c3 = c3 * (frac.x * (1.0 - frac.y));
    c4 = c4 *((1.0 - frac.x) * (1.0 - frac.y));

    let object_color = (c1 + c2 + c3 + c4) * vertex.col;

    if (object_color.a <= 0.0) {
        discard;
    }

    return object_color;
}