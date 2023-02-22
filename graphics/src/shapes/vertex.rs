use crate::InstanceLayout;
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectVertex {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub border_width: f32,
    pub container_data: [f32; 4],
    pub border_data: [f32; 4],
    pub layer: u32,
    pub border_layer: u32,
    pub radius: f32,
}

impl Default for RectVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            size: [0.0; 2],
            border_width: 0.0,
            container_data: [0.0; 4],
            border_data: [0.0; 4],
            layer: 0,
            border_layer: 0,
            radius: 1.0,
        }
    }
}

impl InstanceLayout for RectVertex {
    fn is_bounded() -> bool {
        true
    }

    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Float32, 4 => Float32x4, 5 => Float32x4, 6 => Uint32, 7 => Uint32, 8 => Float32]
            .to_vec()
    }

    ///default set as large enough to contain 1_000 shapes.
    fn default_buffer() -> Vec<u8> {
        Self::with_capacity(1_000)
    }

    fn with_capacity(capacity: usize) -> Vec<u8> {
        let instance_arr: Vec<RectVertex> =
            iter::repeat(RectVertex::default()).take(capacity).collect();

        bytemuck::cast_slice(&instance_arr).to_vec()
    }

    fn instance_stride() -> usize {
        std::mem::size_of::<[f32; 17]>()
    }
}
