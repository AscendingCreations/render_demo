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
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec3<f32>,
    @location(2) color: vec4<u32>,
    @location(3) frames: vec3<u32>,
    @location(4) tex_hw: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
    @location(1) col: vec4<u32>,
    @location(2) frames: vec3<u32>,
    @location(3) tex_hw: vec2<u32>,
    @location(4) size: vec2<f32>,
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
    result.tex_coords = vec3<f32>(vertex.tex_coords.x / fsize.x, vertex.tex_coords.y / fsize.y, vertex.tex_coords.z);
    result.col = vertex.color;
    result.frames = vertex.frames;
    result.tex_hw = vertex.tex_hw;
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
    var coords = vec2<f32>(0.0, 0.0);

    if (vertex.frames[2] > 0u) {
        let id = time.seconds / (f32(vertex.frames[1]) / 1000.0);
        let frame = u32(floor(id % f32(vertex.frames[0])));
        coords = vec2<f32>((f32(frame * vertex.tex_hw[0]) / vertex.size.x) + vertex.tex_coords.x, vertex.tex_coords.y);
    } else {
        coords = vec2<f32>(vertex.tex_coords.x, vertex.tex_coords.y);
    }

    var step = vec2<f32>(0.5, 0.5);
    var tex_pixel = vertex.size * coords - step.xy / 2.0;

    let corner = floor(tex_pixel) + 1.0;
    let frac = min((corner - tex_pixel) * vec2<f32>(2.0, 2.0), vec2<f32>(1.0, 1.0));

    var c1 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, 0.0)) + 0.5) / vertex.size, i32(vertex.tex_coords.z));
    var c2 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(step.x, 0.0)) + 0.5) / vertex.size, i32(vertex.tex_coords.z));
    var c3 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, step.y)) + 0.5) / vertex.size, i32(vertex.tex_coords.z));
    var c4 = textureSample(tex, tex_sample, (floor(tex_pixel + step.xy) + 0.5) / vertex.size, i32(vertex.tex_coords.z));

    c1 = c1 * (frac.x * frac.y);
    c2 = c2 *((1.0 - frac.x) * frac.y);
    c3 = c3 * (frac.x * (1.0 - frac.y));
    c4 = c4 *((1.0 - frac.x) * (1.0 - frac.y));

    let object_color = (c1 + c2 + c3 + c4);
    var color =  hueShift(object_color.rgb, f32(vertex.col.r));
    let ldchange = vertex.col.g / 1000000000u;
    let ldoffset = f32((vertex.col.g % 900000000u) / 1000000u) / 100.0;

    if (ldchange > 0u) {
        color = color + vec3<f32>(ldoffset, ldoffset, ldoffset);
    } else {
        color = color - vec3<f32>(ldoffset, ldoffset, ldoffset);
    }

    color = color * (f32(vertex.col.b) / 100.0);

    let alpha = object_color.a * (f32(vertex.col.a)/ 100.0);

    if (alpha <= 0.0) {
        discard;
    }

    return vec4<f32>(object_color.rgb, alpha);
}