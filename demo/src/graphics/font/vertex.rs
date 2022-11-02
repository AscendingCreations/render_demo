use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// 4 of these per each layer.
pub struct TextVertex {
    pub position: [f32; 3],
    pub tex_coord: [u16; 2],
    pub layer: u32,
    pub color: u32,
}

impl Default for TextVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 1.0],
            tex_coord: [0; 2],
            layer: 0,
            color: 0,
        }
    }
}

impl BufferLayout for TextVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Uint32, 3 => Uint32]
            .to_vec()
    }

    ///default set as large enough to contain 1024 glyphs.
    fn default_buffer() -> BufferPass {
        Self::with_capacity(4096)
    }

    fn with_capacity(capacity: usize) -> BufferPass {
        let vertex_arr: Vec<TextVertex> = iter::repeat(TextVertex::default())
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
