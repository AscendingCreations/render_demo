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
var<uniform> resolution: ScreenResolution;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) dim: vec2<f32>,
    @location(2) uv: vec3<f32>,
    @location(3) color: vec4<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec3<f32>,
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
    let width = vertex.dim.x;
    let height = (vertex.dim.y);
    var uv = vec2<f32>(vertex.uv.x, vertex.uv.y);
    //let v = vertex.vertex_idx % 4u;
    let color = vertex.color;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));

    result.position =  camera.view_proj * vec4<f32>(vertex.pos.xy,1.0, 1.0);

    //result.position = view_proj * vec4<f32>(
    //    pos.x / f32(resolution.width),
    //    pos.y / f32(resolution.height),
    //    1.0,
    //    1.0,
    //);

    //result.position.y *= -1.0;

    result.color = vec4<f32>(
        1.0,
        1.0,
        1.0,
        1.0,
    ) ;

    var uv  = vec2<f32>(uv) /  f32(2048);

    result.uv = vec3<f32>(uv.x, uv.y, vertex.uv.z);
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let color = vertex.color.rgb * textureSample(tex, tex_sample, vertex.uv.xy, i32(vertex.uv.z)).r;
    return vec4<f32>(color, vertex.color.a);
}