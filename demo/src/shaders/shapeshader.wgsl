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
    @location(0) position: vec3<f32>,
    @location(1) size: vec2<f32>,
    @location(2) border_width: f32,
    @location(3) color: u32,
    @location(4) border_color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) border_color: vec4<f32>,
    @location(3) size: vec2<f32>,
    @location(4) border_width: f32,
};

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

    result.color = unpack_color(vertex.color);
    result.border_color = unpack_color(vertex.border_color);
    result.border_width = vertex.border_width;

    var pos = vertex.position.xy;
    switch v {
        case 1u: {
            pos.x += vertex.size.x;
        }
        case 2u: {
            pos += vertex.size;
        }
        case 3u: {
            pos.y += vertex.size.y;
        }
        default: {
        }
    }

    result.clip_position = camera.proj * camera.view * vec4<f32>(pos, vertex.position.z, 1.0);
    result.size = vertex.size * camera.scale;
    result.position = vertex.position.xy * camera.scale;
    return result;
}

fn distance_alg(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    var inner_size: vec2<f32> = size - vec2<f32>(radius, radius) * 2.0;
    var top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    var bottom_right: vec2<f32> = top_left + inner_size;

    var top_left_distance: vec2<f32> =  top_left - frag_coord;
    var bottom_right_distance: vec2<f32> = frag_coord - bottom_right;

    var dist: vec2<f32> = vec2<f32>(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );

    return sqrt(dist.x * dist.x + dist.y * dist.y);
}


// Fragment shader thanks to ICED/hector.
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var mixed_color: vec4<f32> = vertex.color;
    let radius = 5.0;
    let clippy = vec2<f32>(vertex.clip_position.x, screen.size.y - vertex.clip_position.y);

    if (vertex.border_width > 0.0) {
        var border: f32 = max(radius - vertex.border_width, 0.0);

        let distance = distance_alg( 
            clippy, 
            vertex.position.xy + vec2<f32>(vertex.border_width), 
            vertex.size - vec2<f32>(vertex.border_width * 2.0), 
            border 
        );

        let border_mix: f32 = smoothstep(
            max(border - 0.5, 0.0),
            border + 0.5,
            distance
        );

        mixed_color = mix(vertex.color, vertex.border_color, vec4<f32>(border_mix));
    }

    let dist: f32 = distance_alg(
        clippy,
        vertex.position.xy,
        vertex.size,
        radius
    );

    let radius_alpha: f32 = 1.0 - smoothstep(
        max(radius - 0.5, 0.0),
        radius + 0.5,
        dist);

    return vec4<f32>(mixed_color.r, mixed_color.g, mixed_color.b, mixed_color.a * radius_alpha);
}