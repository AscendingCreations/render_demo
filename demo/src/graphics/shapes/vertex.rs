use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub border_width: f32,
    pub color: u32,
    pub border_color: u32,
}

impl Default for ShapeVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            size: [0.0; 2],
            border_width: 0.0,
            color: 0,
            border_color: 0,
        }
    }
}

impl BufferLayout for ShapeVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32, 3 => Uint32, 4 => Uint32].to_vec()
    }

    fn default_buffer() -> BufferPass {
        Self::with_capacity(1_000)
    }

    fn with_capacity(capacity: usize) -> BufferPass {
        let vertex_arr: Vec<ShapeVertex> = iter::repeat(ShapeVertex::default())
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
        std::mem::size_of::<[f32; 8]>()
    }

    fn index_stride() -> usize {
        4
    }

    fn index_offset() -> usize {
        6
    }
}
