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
    @location(0) v_pos: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) size: vec2<f32>,
    @location(3) border_width: f32,
    @location(4) container_data: vec2<u32>,
    @location(5) border_data: vec2<u32>,
    @location(6) layer: u32,
    @location(7) border_layer: u32,
    @location(8) radius: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
    @location(1) container_uv: vec2<f32>,
    @location(2) border_uv: vec2<f32>,
    @location(3) container_data: vec4<u32>,
    @location(4) border_data: vec4<u32>,
    @location(5) size: vec2<f32>,
    @location(6) border_width: f32,
    @location(7) radius: f32,
    @location(8) layer: i32,
    @location(9) border_layer: i32,
    @location(10) tex_size: vec2<f32>,
};

@group(1)
@binding(0)
var tex: texture_2d_array<f32>;
@group(1)
@binding(1)
var tex_sample: sampler;

fn unpack_tex_data(data: vec2<u32>) -> vec4<u32> {
    return vec4<u32>(
        u32(data[0] & 0xffffu), 
        u32((data[0] & 0xffff0000u) >> 16u),
        u32(data[1] & 0xffffu),
        u32((data[1] & 0xffff0000u) >> 16u)
    );
}

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    let v = vertex.vertex_idx % 4u;
    let tex_data = unpack_tex_data(vertex.container_data);
    let bor_data = unpack_tex_data(vertex.border_data);
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));
    var pos = vertex.position;

     switch v {
        case 1u: {
            result.container_uv = vec2<f32>(f32(tex_data[2]), f32(tex_data[3]));
            result.border_uv = vec2<f32>(f32(bor_data[2]), f32(bor_data[3]));
            pos.x += vertex.size.x;
        }
        case 2u: {
            result.container_uv = vec2<f32>(f32(tex_data[2]), 0.0);
            result.border_uv = vec2<f32>(f32(bor_data[2]), 0.0);
            pos.x += vertex.size.x;
            pos.y += vertex.size.y;
        }
        case 3u: {
            result.container_uv = vec2<f32>(0.0, 0.0);
            result.border_uv = vec2<f32>(0.0, 0.0);
            pos.y += vertex.size.y;
        }
        default: {
            result.container_uv = vec2<f32>(0.0, f32(tex_data[3]));
            result.border_uv = vec2<f32>(0.0, f32(bor_data[3]));
        }
    }

    result.clip_position = camera.proj * vec4<f32>(pos, 1.0);
    result.container_data = tex_data;
    result.border_data = bor_data;
    result.border_width = vertex.border_width;
    result.size = vertex.size;
    result.position = vertex.position.xy;
    result.radius = vertex.radius;
    result.tex_size = fsize;
    result.border_layer = i32(vertex.border_layer);
    result.layer = i32(vertex.layer);
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
    let container_coords = vec2<f32>(
        (f32(vertex.container_data[0]) + vertex.container_uv.x) / vertex.tex_size.x,
        (f32(vertex.container_data[1]) + vertex.container_uv.y) / vertex.tex_size.y
    );
        
    let border_coords = vec2<f32>(
        (f32(vertex.border_data[0]) + vertex.border_uv.x) / vertex.tex_size.x,
        (f32(vertex.border_data[1]) + vertex.border_uv.y) / vertex.tex_size.y
    );

    let border_color = textureSampleLevel(tex, tex_sample, border_coords, vertex.border_layer, 1.0);
    let container_color = textureSampleLevel(tex, tex_sample, container_coords, vertex.layer, 1.0);

    var mixed_color: vec4<f32> = container_color;
    let radius = vertex.radius;
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

        mixed_color = mix(container_color, border_color, vec4<f32>(border_mix));
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