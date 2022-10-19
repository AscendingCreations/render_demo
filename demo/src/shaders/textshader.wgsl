struct Camera {
    view_proj: mat4x4<f32>,
    eye: vec3<f32>,
};

struct Time {
    seconds: f32,
};

struct ScreenResolution {
    width: u32,
    height: u32
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> time: Time;

@group(2)
@binding(0)
var<uniform> resolution: vec2<u32>;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: u32,
    @location(2) layer: u32,
    @location(3) color: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) layer: i32,
};

@group(3)
@binding(0)
var tex: texture_2d_array<f32>;
@group(3)
@binding(1)
var tex_sample: sampler;

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    var pos = vertex.pos;
    let u = vertex.uv & 0xffffu;
    let v = (vertex.uv & 0xffff0000u) >> 16u;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));

    //result.position = matix * vec4<f32>(vertex.pos.xyz, 1.0);
    result.position = vec4<f32>(
        2.0 * vec2<f32>(vertex.pos.xy) / vec2<f32>(resolution) - 1.0,
       vertex.pos.z,
        1.0,
    );

    result.size = fsize;
    result.color = vec4<f32>(
        f32((vertex.color & 0xffu)),
        f32((vertex.color & 0xff00u) >> 8u),
        f32((vertex.color & 0xff0000u) >> 16u),
        f32((vertex.color & 0xff000000u) >> 24u),
    ) / 255.0;

    result.uv = vec2<f32>(f32(u), f32(v)) /  fsize;
    result.layer = i32(vertex.layer);
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