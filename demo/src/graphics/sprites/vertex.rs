pub(crate) use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub tex_coord: [u16; 2],
    pub rg: [u32; 2],
    pub ba: [u16; 2],
    pub frames: [u16; 2],
    pub tex_hw: [u16; 2],
    pub time: u32,
    pub layer: i32,
}

impl Default for SpriteVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0; 2],
            rg: [0; 2],
            ba: [0; 2],
            frames: [0; 2],
            tex_hw: [0; 2],
            time: 0,
            layer: 0,
        }
    }
}

impl BufferLayout for SpriteVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Uint32x2, 3 => Uint32, 4 => Uint32, 5 => Uint32, 6 => Uint32, 7 => Sint32  ]
            .to_vec()
    }

    fn default_buffer() -> BufferPass {
        Self::with_capacity(10_000)
    }

    fn with_capacity(capacity: usize) -> BufferPass {
        let vertex_arr: Vec<SpriteVertex> =
            iter::repeat(SpriteVertex::default())
                .take(capacity * 4)
                .collect();

        let mut indices: Vec<u32> = Vec::with_capacity(capacity * 6);

        (0..capacity as u32).for_each(|i| {
            let x = i * 4;
            indices.extend_from_slice(&[x, x + 1, x + 2, x, x + 2, x + 3]);
        });

        BufferPass {
            vertices: bytemuck::cast_slice(&vertex_arr).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }

    fn vertex_stride() -> usize {
        std::mem::size_of::<[f32; 11]>()
    }

    fn index_stride() -> usize {
        4
    }

    fn index_offset() -> usize {
        6
    }
}
