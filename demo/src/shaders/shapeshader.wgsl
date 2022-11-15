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
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) col: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    let v = vertex.vertex_idx % 4u;
    result.clip_position =  (camera.proj * camera.view) * vec4<f32>(vertex.position.xyz, 1.0);
    result.col = vec4<f32>(
        f32((vertex.color & 0xffu)),
        f32((vertex.color & 0xff00u) >> 8u),
        f32((vertex.color & 0xff0000u) >> 16u),
        f32((vertex.color & 0xff000000u) >> 24u),
    ) / 255.0;
    
    switch v {
        case 1u: {
            result.uv = vec2<f32>(1.0, 0.0);
        }
        case 2u: {
            result.uv = vec2<f32>(1.0, 1.0);
        }
        case 3u: {
            result.uv = vec2<f32>(0.0, 1.0);
        }
        default: {
            result.uv = vec2<f32>(0.0, 0.0);
        }
    }

    return result;
}

fn circle(pos: vec2<f32>, radius: f32) -> vec2<f32> {
    let v = pos - vec2<f32>(0.5);
    let dist = sqrt(dot(v, v));
                       // leh border                                        //leh fill
    return vec2<f32>(1.0 - smoothstep(radius - 0.04, radius, dist), 1.0 - smoothstep(radius, radius + 0.04, dist));
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let circle = circle(vertex.uv, 0.45);

    let border = vec3<f32>(1.0,1.0,1.0);
    let fill = vec3<f32>(0.0,0.0,0.0);
    return vec4<f32>(mix(border.rgb, fill.rgb, circle.x), circle.y);
}