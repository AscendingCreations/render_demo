use crate::{BufferData, BufferLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: u32,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            uv: [0.0; 2],
            color: 0,
        }
    }
}

impl BufferLayout for MeshVertex {
    fn is_bounded() -> bool {
        true
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Uint32]
            .to_vec()
    }

    //default set as large enough to contain 1_000 vertices.
    fn default_buffer() -> BufferData {
        Self::with_capacity(1_000, 6_000)
    }

    fn with_capacity(
        vertex_capacity: usize,
        index_capacity: usize,
    ) -> BufferData {
        let vbo_arr: Vec<MeshVertex> = iter::repeat(MeshVertex::default())
            .take(vertex_capacity)
            .collect();

        let mut indices: Vec<u32> = Vec::with_capacity(index_capacity * 6);
        (0..index_capacity as u32).for_each(|_| {
            indices.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        });

        BufferData {
            vertexs: bytemuck::cast_slice(&vbo_arr).to_vec(),
            indexs: bytemuck::cast_slice(&indices).to_vec(),
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 6]>()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshInstance {
    pub layer: u32,
    pub image_uv: [f32; 4],
}

impl Default for MeshInstance {
    fn default() -> Self {
        Self {
            layer: 0,
            image_uv: [0.0; 4],
        }
    }
}

impl BufferLayout for MeshInstance {
    fn is_bounded() -> bool {
        true
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Uint32, 2 => Float32x4].to_vec()
    }

    //default set as large enough to contain 1_000 vertices.
    fn default_buffer() -> BufferData {
        Self::with_capacity(1_000, 0)
    }

    fn with_capacity(
        vertex_capacity: usize,
        index_capacity: usize,
    ) -> BufferData {
        let instance_arr: Vec<MeshInstance> =
            iter::repeat(MeshInstance::default())
                .take(vertex_capacity)
                .collect();

        BufferData {
            vertexs: bytemuck::cast_slice(&instance_arr).to_vec(),
            ..Default::default()
        }
    }

    fn stride() -> usize {
        std::mem::size_of::<[f32; 5]>()
    }
}
