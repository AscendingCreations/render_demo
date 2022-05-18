struct Camera {
    view_proj: mat4x4<f32>,
    eye: vec3<f32>,
};

struct Time {
    seconds: f32,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> time: Time;

struct VertexInput {
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_data: vec4<u32>,
    @location(2) position: vec3<f32>,
    @location(3) hue_alpha: vec2<u32>,
    @location(4) frames: vec3<u32>,
    @location(5) layer: i32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_data: vec4<u32>,
    @location(2) hue_alpha: vec2<u32>,
    @location(3) frames: vec3<u32>,
    @location(5) layer: i32,
    @location(6) size: vec2<f32>,
};

@group(2)
@binding(0)
var tex: texture_2d_array<f32>;
@group(2)
@binding(1)
var tex_sample: sampler;

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));

    result.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    result.tex_coords = vec2<f32>(vertex.tex_coords.x / fsize.x, vertex.tex_coords.y / fsize.y);
    result.tex_data = vertex.tex_data;
    result.hue_alpha = vertex.hue_alpha;
    result.frames = vertex.frames;
    result.layer = vertex.layer;
    result.size = fsize;

    return result;
}

fn hueShift(color: vec3<f32>, hue: f32) -> vec3<f32>
{
    var pi = 3.14159;
    let rad = hue * (pi/180.0);
    let k = vec3<f32>(0.57735, 0.57735, 0.57735);
    let cosAngle = cos(rad);
    return vec3<f32>(color * cosAngle + cross(k, color) * sin(rad) + k * dot(k, color) * (1.0 - cosAngle));
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let id = time.seconds / (f32(vertex.frames[2]) / 1000.0);
    let frame = u32(floor(id % f32(vertex.frames[0])));
    var yframes = vertex.frames[0];

    if (vertex.frames[1] > 0u) {
        yframes = vertex.frames[1];
    }

    let coords = vec2<f32>(
        ((f32(((frame % yframes) * vertex.tex_data[2]) + vertex.tex_data[0]) / vertex.size.x) + vertex.tex_coords.x  + (.5 / f32(vertex.size.x))),
        ((f32(((frame / yframes) * vertex.tex_data[3]) + vertex.tex_data[1]) / vertex.size.y) + vertex.tex_coords.y  + (.5 / f32(vertex.size.x)))
    );

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

    let object_color = (c1 + c2 + c3 + c4);
    let alpha = object_color.a * (f32(vertex.hue_alpha[1]) / 100.0);

    if (alpha <= 0.0) {
        discard;
    }

    let color = hueShift(object_color.rgb, f32(vertex.hue_alpha[0]) % 361.0);
    return vec4<f32>(color, alpha);
}

