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
        let mut stride = std::mem::size_of::<[f32; 2]>();
        stride += std::mem::size_of::<[f32; 4]>();
        stride += std::mem::size_of::<[f32; 3]>();
        stride += std::mem::size_of::<[u32; 2]>();
        stride += std::mem::size_of::<[u32; 3]>();
        stride += std::mem::size_of::<i32>();

        stride
    }

    /// Calculates the offset of each vertex attribute.
    pub fn offsets() -> Vec<usize> {
        let mut offsets = vec![];
        let mut offset = 0;

        offset += std::mem::size_of::<[f32; 2]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[u32; 4]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[f32; 3]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[u32; 2]>();
        offsets.push(offset);
        offset += std::mem::size_of::<[u32; 3]>();
        offsets.push(offset);
        offset += std::mem::size_of::<i32>();
        offsets.push(offset);

        offsets
    }

    pub fn attributes() -> Vec<wgpu::VertexAttribute> {
        let attributes = vec![
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as u64,
                shader_location: 1,
                format: wgpu::VertexFormat::Uint32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 6]>() as u64,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 9]>() as u64,
                shader_location: 3,
                format: wgpu::VertexFormat::Uint32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 11]>() as u64,
                shader_location: 4,
                format: wgpu::VertexFormat::Uint32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 14]>() as u64,
                shader_location: 5,
                format: wgpu::VertexFormat::Sint32,
            },
        ];

        attributes
    }
}
