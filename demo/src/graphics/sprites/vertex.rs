#[derive(Clone, Debug)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 3],
    pub color: [f32; 4],
}

impl Default for SpriteVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tex_coord: [0.0, 0.0, 1.0],
            color: [1.0; 4],
        }
    }
}

impl SpriteVertex {
    pub fn to_bytes(&self, bytes: &mut Vec<u8>) {
        let slice: [f32; 3] = self.position;
        bytes.extend_from_slice(bytemuck::cast_slice(&slice));

        let slice: [f32; 3] = self.tex_coord;
        bytes.extend_from_slice(bytemuck::cast_slice(&slice));

        let slice: [f32; 4] = self.color;
        bytes.extend_from_slice(bytemuck::cast_slice(&slice));
    }

    /// Calculate the stride between two vertices in bytes, i.e. how large each vertex is in bytes.
    pub fn stride() -> usize {
        let mut stride = std::mem::size_of::<[f32; 3]>();
        stride += std::mem::size_of::<[f32; 3]>();
        stride += std::mem::size_of::<[f32; 4]>();

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
        offset += std::mem::size_of::<[f32; 4]>();
        offsets.push(offset);

        offsets
    }

    pub fn attributes() -> Vec<wgpu::VertexAttribute> {
        let mut attributes = vec![];
        let mut offset = 0;

        // Add the vertex attribute to hold the position.
        attributes.push(wgpu::VertexAttribute {
            offset: offset as wgpu::BufferAddress,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        });

        offset += std::mem::size_of::<[f32; 3]>();

        attributes.push(wgpu::VertexAttribute {
            offset: offset as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3,
        });

        offset += std::mem::size_of::<[f32; 3]>();

        attributes.push(wgpu::VertexAttribute {
            offset: offset as wgpu::BufferAddress,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        });

        attributes
    }
}
