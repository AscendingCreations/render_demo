use crate::{BufferData, BufferLayout};
use cosmic_text::Color;
use lyon::{math::Point as LPoint, tessellation as tess};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub color: u32,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            color: 0,
        }
    }
}

impl BufferLayout for MeshVertex {
    fn is_bounded() -> bool {
        true
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32].to_vec()
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
        std::mem::size_of::<[f32; 4]>()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct VertexBuilder {
    pub z: f32,
    pub color: Color,
}

impl VertexBuilder {
    pub fn new_vertex(self, position: LPoint) -> MeshVertex {
        MeshVertex {
            position: [position.x, position.y, self.z],
            color: self.color.0,
        }
    }
}

impl tess::StrokeVertexConstructor<MeshVertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: tess::StrokeVertex) -> MeshVertex {
        let position = vertex.position();
        MeshVertex {
            position: [position.x, position.y, self.z],
            color: self.color.0,
        }
    }
}

impl tess::FillVertexConstructor<MeshVertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: tess::FillVertex) -> MeshVertex {
        let position = vertex.position();
        MeshVertex {
            position: [position.x, position.y, self.z],
            color: self.color.0,
        }
    }
}
