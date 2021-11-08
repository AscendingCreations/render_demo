pub(crate) use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 3],
    pub color: [u32; 4],
    pub frames: [u32; 3],
    pub tex_hw: [u32; 2],
}

impl Default for SpriteVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0; 3],
            color: [0; 4],
            frames: [0; 3],
            tex_hw: [0; 2],
        }
    }
}

impl BufferLayout for SpriteVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32x4, 3 => Uint32x3, 4 => Uint32x2]
            .to_vec()
    }

    fn initial_buffer() -> BufferPass {
        let vertex_arr: Vec<SpriteVertex> = iter::repeat(SpriteVertex {
            position: [0.0; 3],
            tex_coord: [0.0; 3],
            color: [0; 4],
            frames: [0; 3],
            tex_hw: [0; 2],
        })
        .take(40_000)
        .collect();

        let mut indices: Vec<u32> = Vec::with_capacity(60_000);

        (0..10_000).for_each(|i| {
            let x = i * 4;
            indices.extend_from_slice(&[x, x + 1, x + 2, x, x + 2, x + 3]);
        });

        BufferPass {
            vertices: bytemuck::cast_slice(&vertex_arr).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }

    fn stride() -> u64 {
        std::mem::size_of::<[f32; 15]>() as u64
    }
}
