pub(crate) use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// 4 of these per each layer.
pub struct TextVertex {
    pub position: [f32; 2],
    pub dimension: [f32; 2],
    pub tex_coord: [f32; 3],
    pub color: [u8; 4],
}

impl Default for TextVertex {
    fn default() -> Self {
        Self {
            position: [0; 2],
            dimension: [0; 2],
            tex_coord: [0; 3],
            color: [0; 4],
        }
    }
}

impl BufferLayout for TextVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x3, 3 => Uint8x4].to_vec()
    }

    ///default set as large enough to contain 90 maps with all layers
    fn default_buffer() -> BufferPass {
        Self::with_capacity(720)
    }

    fn with_capacity(capacity: usize) -> BufferPass {
        let vertex_arr: Vec<TextVertex> = iter::repeat(TextVertex {
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            tex_coord: [0.0, 0.0, 0.0],
            color: [0, 0, 0, 0],
        })
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
        std::mem::size_of::<[f32; 6]>()
    }

    fn index_stride() -> usize {
        4
    }

    fn index_offset() -> usize {
        6
    }
}
