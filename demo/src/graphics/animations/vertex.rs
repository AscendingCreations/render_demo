use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnimationVertex {
    pub tex_coord: [f32; 2],
    pub tex_data: [u32; 4],
    pub position: [f32; 3],
    pub hue_alpha: [u32; 2],
    pub frames: [u32; 3],
    pub layer: i32,
}

impl Default for AnimationVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0; 2],
            tex_data: [0; 4],
            layer: 0,
            hue_alpha: [0; 2],
            frames: [0; 3],
        }
    }
}

impl BufferLayout for AnimationVertex {
    fn stride() -> u64 {
        std::mem::size_of::<[f32; 15]>() as u64
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32x4, 2 => Float32x3, 3 => Uint32x2, 4 => Uint32x3, 5 => Sint32].to_vec()
    }

    fn initial_buffer() -> BufferPass {
        let vertex_arr: Vec<AnimationVertex> = iter::repeat(AnimationVertex {
            tex_coord: [0.0; 2],
            tex_data: [0; 4],
            position: [0.0; 3],
            hue_alpha: [0; 2],
            frames: [0; 3],
            layer: 0,
        })
        .take(40_000)
        .collect();

        let mut indices: Vec<u32> = Vec::with_capacity(60_000);

        for i in 0..10_000 {
            let x = i * 4;
            indices.extend_from_slice(&[x, x + 1, x + 2, x, x + 2, x + 3]);
        }

        BufferPass {
            vertices: bytemuck::cast_slice(&vertex_arr).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }
}
