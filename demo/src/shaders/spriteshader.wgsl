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
    [[location(2)]] color: vec4<u32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec3<f32>;
    [[location(1)]] col: vec4<u32>;
};

[[stage(vertex)]]
fn main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.col = vertex.color;
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
    let coords = vec3<i32>(i32(in.tex_coords.x), i32(in.tex_coords.y), i32(in.tex_coords.z));
    let object_color = textureLoad(tex, coords.xy, coords.z, 0);
    var color =  hueShift(object_color.rgb, f32(in.col.r));
    let ldchange = in.col.g / 1000000000u;
    let ldoffset = f32((in.col.g % 900000000u) / 1000000u) / 100.0;

    if (ldchange > 0u) {
        color = color + vec3<f32>(ldoffset, ldoffset, ldoffset);
    } else {
        color = color - vec3<f32>(ldoffset, ldoffset, ldoffset);
    }

    color = color * (f32(in.col.b) / 100.0);
    let alpha = object_color.a * (f32(in.col.a)/ 100.0);
    return vec4<f32>(object_color.rgb, alpha);
}