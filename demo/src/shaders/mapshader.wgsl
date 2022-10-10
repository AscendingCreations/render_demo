struct Camera {
    view_proj: mat4x4<f32>,
    eye: vec3<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
    @location(1) zpos: f32,
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

    result.zpos = vertex.position.z;
    result.clip_position =  camera.view_proj * vec4<f32>(vertex.position.xyz, 1.0);
    result.tex_coords = vertex.tex_coords.xyz;
    return result;
}

@group(3)
@binding(0)
var maptex: texture_2d_array<u32>;

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
    let yoffset = abs((i32(vertex.zpos) - 8) * 32);
    let coords = vec3<i32> (i32(vertex.tex_coords.x + .5), i32(vertex.tex_coords.y + .5), i32(vertex.tex_coords.z));
    let tile_pos = vec2<i32>(coords.x / 16, (coords.y / 16) + yoffset);
    let tile = textureLoad(maptex, tile_pos.xy, coords.z, 0);
    let pos = vec2<i32>(i32(tile.r % 128u) * 16 + (coords.x % 16), i32(tile.r / 128u) * 16 + (coords.y % 16));
    let object_color = textureLoad(tex, pos.xy, i32(tile.g), 0);
    let alpha = object_color.a * (f32(tile.a) / 100.0);

    if (alpha <= 0.0) {
        discard;
    }

    let color = hueShift(object_color.rgb, f32(tile.b) % 361.0);
    return vec4<f32>(color, alpha);
}

