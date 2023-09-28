struct Global {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    eye: vec3<f32>,
    scale: f32,
    size: vec2<f32>,
    seconds: f32,
};

struct AreaLights {
    pos: vec2<f32>,
    color: u32,
    max_distance: f32,
    animate: u32,
};


struct DirLights {
    pos: vec2<f32>,
    color: u32,
    max_distance: f32,
    max_radius: f32,
    smoothness: f32,
    angle: f32,
    animate: u32,
};

@group(0)
@binding(0)
var<uniform> global: Global;

struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) world_color: u32,
    @location(1) enable_lights: u32,
    @location(2) dir_count: u32,
    @location(3) area_count: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) col: vec4<f32>,
    @location(2) enable_lights: u32,
    @location(3) dir_count: u32,
    @location(4) area_count: u32,
};

const c_area_lights: u32 = 2000u;
const c_dir_lights: u32 = 2000u;

@group(1)
@binding(0)
var<uniform> u_areas: array<AreaLights, 2000>;
@group(1)
@binding(1)
var<uniform> u_dirs: array<DirLights, 2000>;

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
    let tex_data = vertex.tex_data;

    switch v {
        case 1u: {
            result.tex_coords = vec2<f32>(1.0, 1.0);
            result.clip_position = vec4<f32>(1.0, 0.0, 1.0, 1.0);
        }
        case 2u: {
            result.tex_coords = vec2<f32>(1.0, 0.0);
            result.clip_position = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
        case 3u: {
            result.tex_coords = vec2<f32>(0.0, 0.0);
            result.clip_position = vec4<f32>(0.0, 1.0, 1.0, 1.0);
            pos.y += vertex.hw.y;
        }
        default: {
            result.tex_coords = vec2<f32>(0.0, 1.0);
            result.clip_position = vec4<f32>(0.0, 0.0, 1.0, 1.0);
        }
    }

    result.col = unpack_color(vertex.world_color);
    result.enable_lights = vertex.enable_lights;
    result.dir_count = vertex.dir_count;
    result.area_count = vertex.area_count;
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    var col = vertex.col;

    if (vertex.enable_lights > 0u) {
        for(var i = 0u; i < min(vetex.area_count, c_area_lights); i += 1u) {
            let light = u_areas[i];
            let pos = (global.proj * global.view) * vec4<f32>(light.pos.x, light.pos.y, 1.0, 1.0);

            var max_distance = light.max_distance;

            if (light.animate > 0u) {
                max_distance = light.max_distance - 0.015 * sin(global.seconds);
            }
    
            let dist = distance(vertex.tex_coords, pos.xy);
            let value = 1.0 - smoothstep(0.1, max_distance, dist);

            col = mix(col, light.col, vec4<f32>(value));
        }
    } 

    if (col.a <= 0.0) {
        discard;
    }

    return col;
}