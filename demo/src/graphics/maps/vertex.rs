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

impl MapVertex {
    /// Calculate the stride between two vertices in bytes, i.e. how large each vertex is in bytes.
    pub fn stride() -> usize {
        std::mem::size_of::<[f32; 6]>()
    }

    pub fn attributes() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3].to_vec()
    }
}
