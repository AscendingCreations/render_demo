#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 3],
    pub color: [u32; 4],
}

impl Default for SpriteVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0; 3],
            color: [0; 4],
        }
    }
}

impl SpriteVertex {
    /// Calculate the stride between two vertices in bytes, i.e. how large each vertex is in bytes.
    pub fn stride() -> usize {
        let mut stride = std::mem::size_of::<[f32; 3]>();
        stride += std::mem::size_of::<[f32; 3]>();
        stride += std::mem::size_of::<[u32; 4]>();

        stride
    }

    /// Calculates the offset of each vertex attribute.
    pub fn offsets() -> Vec<usize> {
        let mut offsets = vec![];
        let mut offset = 0;

        offset += std::mem::size_of::<[f32; 3]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[f32; 3]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[u32; 4]>();
        offsets.push(offset);

        offsets
    }

    pub fn attributes() -> Vec<wgpu::VertexAttribute> {
        let attributes = vec![
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as u64,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 6]>() as u64,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32x4,
            },
        ];

        attributes
    }
}
