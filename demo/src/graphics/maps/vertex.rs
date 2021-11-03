use crate::graphics::{BufferLayout, BufferPass};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// 4 of these per each layer.
pub struct MapVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 3],
}

impl Default for MapVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0; 3],
        }
    }
}

impl BufferLayout for MapVertex {
    fn stride() -> u64 {
        std::mem::size_of::<[f32; 6]>() as u64
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3].to_vec()
    }

    fn initial_buffer() -> BufferPass {
        let vertex_arr: Vec<MapVertex> = iter::repeat(MapVertex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0, 0.0],
        })
        .take(2_880)
        .collect();

        let mut indices: Vec<u32> = Vec::with_capacity(4_320);

        for i in 0..720 {
            let x = i * 4;
            indices.extend_from_slice(&[x, x + 1, x + 2, x, x + 2, x + 3]);
        }

        BufferPass {
            vertices: bytemuck::cast_slice(&vertex_arr).to_vec(),
            indices: bytemuck::cast_slice(&indices).to_vec(),
        }
    }
}
