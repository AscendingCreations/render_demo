#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnimationVertex {
    pub tex_coord: [f32; 2],
    pub tex_data: [u32; 4],
    pub position: [f32; 3],
    pub hue_alpha: [u32; 2],
    pub frames: [u32; 3],
    pub layer: i32,
}

impl Default for AnimationVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0; 2],
            tex_data: [0; 4],
            layer: 0,
            hue_alpha: [0; 2],
            frames: [0; 3],
        }
    }
}

impl AnimationVertex {
    /// Calculate the stride between two vertices in bytes, i.e. how large each vertex is in bytes.
    pub fn stride() -> usize {
        std::mem::size_of::<[f32; 15]>()
    }

    pub fn attributes() -> Vec<wgpu::VertexAttribute> {
        // Use their macro to reduce the code we need to write.
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32x4, 2 => Float32x3, 3 => Uint32x2, 4 => Uint32x3, 5 => Sint32].to_vec()
    }
}
