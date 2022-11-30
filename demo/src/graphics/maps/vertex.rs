use crate::graphics::{BufferLayout, BufferPass, InstanceLayout};
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// 4 of these per each layer.
pub struct MapVertex {
    pub position: [f32; 3],
    pub hw: [f32; 2],
    pub layer: i32,
}

impl Default for MapVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            hw: [0.0; 2],
            layer: 0,
        }
    }
}

impl InstanceLayout for MapVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Sint32]
            .to_vec()
    }

    ///default set as large enough to contain 720 map layers.
    fn default_buffer() -> Vec<u8> {
        Self::with_capacity(720)
    }

    fn with_capacity(capacity: usize) -> Vec<u8> {
        let instance_arr: Vec<MapVertex> =
            iter::repeat(MapVertex::default()).take(capacity).collect();

        bytemuck::cast_slice(&instance_arr).to_vec()
    }

    fn instance_stride() -> usize {
        std::mem::size_of::<[f32; 6]>()
    }
}
