struct Global {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inverse_proj: mat4x4<f32>,
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
    padding: vec3<f32>,
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
    @location(0) v_pos: vec2<f32>,
    @location(1) world_color: vec4<f32>,
    @location(2) enable_lights: u32,
    @location(3) dir_count: u32,
    @location(4) area_count: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec4<f32>,
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
@group(2)
@binding(0)
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

    switch v {
        case 1u: {
            result.clip_position = global.proj * vec4<f32>(global.size.x, 0.0, 1.0, 1.0);
        }
        case 2u: {
            result.clip_position = global.proj * vec4<f32>(global.size.x, global.size.y, 1.0, 1.0);
        }
        case 3u: {
            result.clip_position = global.proj * vec4<f32>(0.0, global.size.y, 1.0, 1.0);
        }
        default: {
            result.clip_position = global.proj * vec4<f32>(0.0, 0.0, 1.0, 1.0);
        }
    }

    result.tex_coords = global.inverse_proj * result.clip_position;
    result.tex_coords = result.tex_coords / result.tex_coords.w;
    result.col = vertex.world_color;
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
        //for(var i = 0u; i < min(vertex.area_count, c_area_lights); i += 1u) {
            let light = u_areas[0];
            let pos = vec4<f32>(light.pos.x, light.pos.y, 1.0, 1.0);

            var max_distance = light.max_distance;

            if (light.animate > 0u) {
                max_distance = light.max_distance - 0.015 * sin(global.seconds);
            }
    
            let dist = distance(vertex.tex_coords.xy, pos.xy);
            let d = min(1.0, dist / max_distance);
            let value = pow(1.0 - pow(d, 2.0), 2.0) / (1.0 + pow(d, 2.0));
            //let value2 = clamp(value, 0.0, 1.0);
            let color2 = col; 

            col = mix(color2,unpack_color(light.color), vec4<f32>(value));
       // }
    } 

    if (col.a <= 0.0) {
        discard;
    }

    return col;
}