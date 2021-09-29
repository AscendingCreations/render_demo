#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
//4 of these per each layer.
pub struct MapVertex {
    pub position: [f32; 3],
    pub tex_pos: [f32; 2],
}

impl Default for MapVertex {
    fn default() -> Self {
        Self { position: [0.0; 3]
            position: [0.0; 3]}
    }
}

impl MapVertex {
    /// Calculate the stride between two vertices in bytes, i.e. how large each vertex is in bytes.
    pub fn stride() -> usize {
        let mut stride = std::mem::size_of::<[f32; 3]>();
        stride += std::mem::size_of::<[f32; 2]>();

        stride
    }

    /// Calculates the offset of each vertex attribute.
    pub fn offsets() -> Vec<usize> {
        let mut offsets = vec![];
        let mut offset = 0;

        offset += std::mem::size_of::<[f32; 3]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[f32; 2]>();
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
                format: wgpu::VertexFormat::Float32x2,
            },
        ];

        attributes
    }
}
