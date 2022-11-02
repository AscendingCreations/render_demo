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
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) col: vec4<f32>,
};

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;

    result.clip_position =  (camera.proj * camera.view) * vec4<f32>(vertex.position.xyz, 1.0);
    result.col = vec4<f32>(
        f32((vertex.color & 0xffu)),
        f32((vertex.color & 0xff00u) >> 8u),
        f32((vertex.color & 0xff0000u) >> 16u),
        f32((vertex.color & 0xff000000u) >> 24u),
    ) / 255.0;

    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    if (vertex.col.a <= 0.0) {
        discard;
    }

    return vertex.col;
}