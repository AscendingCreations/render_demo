use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 3],
    pub color: [u32; 4],
}

impl Default for SpriteVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0; 3],
            color: [0; 4],
        }
    }
}

impl BufferLayout for SpriteVertex {
    fn stride() -> u64 {
        std::mem::size_of::<[f32; 10]>() as u64
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32x4].to_vec()
    }

    fn initial_buffer() -> BufferPass {
        let vertex_arr: Vec<SpriteVertex> = iter::repeat(SpriteVertex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0, 0.0],
            color: [0, 0, 0, 0],
        })
        .take(40_000)
        .collect();

        let indices = (0..10_000)
            .map(|x| vec![x, x + 1, x + 2, x, x + 2, x + 3])
            .flatten()
            .collect::<Vec<u32>>();

        BufferPass {
            vertices: bytemuck::cast_slice(&vertex_arr).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }
}
