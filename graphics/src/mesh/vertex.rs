use crate::InstanceLayout;
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub uv: [f32; 3],
    pub color: u32,
}

impl Default for RectVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            uv: [0.0; 3],
            color: 0,
        }
    }
}

impl InstanceLayout for RectVertex {
    fn is_bounded() -> bool {
        true
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x3, 3 => Float32]
            .to_vec()
    }

    ///default set as large enough to contain 1_000 vertices.
    fn default_buffer() -> Vec<u8> {
        Self::with_capacity(1_000)
    }

    fn with_capacity(capacity: usize) -> Vec<u8> {
        let instance_arr: Vec<RectVertex> =
            iter::repeat(RectVertex::default()).take(capacity).collect();

        bytemuck::cast_slice(&instance_arr).to_vec()
    }

    fn instance_stride() -> usize {
        std::mem::size_of::<[f32; 7]>()
    }
}
