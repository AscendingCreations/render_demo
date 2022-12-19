use crate::InstanceLayout;
use std::iter;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub hw: [f32; 2],
    pub tex_data: [u16; 4],
    pub color: u32,
    pub frames: [u16; 2],
    pub animate: u32,
    pub time: u32,
    pub layer: i32,
}

impl Default for SpriteVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            hw: [0.0; 2],
            tex_data: [0; 4],
            color: 0,
            frames: [0; 2],
            animate: 0,
            time: 0,
            layer: 0,
        }
    }
}

impl InstanceLayout for SpriteVertex {
    fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![1 => Float32x3, 2 => Float32x2, 3 => Uint32x2, 4 => Uint32, 5 => Uint32, 6 => Uint32, 7 => Uint32, 8 => Sint32 ]
            .to_vec()
    }

    ///default set as large enough to contain 10_000 sprites.
    fn default_buffer() -> Vec<u8> {
        Self::with_capacity(10_000)
    }

    fn with_capacity(capacity: usize) -> Vec<u8> {
        let instance_arr: Vec<SpriteVertex> =
            iter::repeat(SpriteVertex::default())
                .take(capacity)
                .collect();

        bytemuck::cast_slice(&instance_arr).to_vec()
    }

    fn instance_stride() -> usize {
        std::mem::size_of::<[f32; 12]>()
    }
}
