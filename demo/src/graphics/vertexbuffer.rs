use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct VertexBufferBuilder<V: VertexBufferExt> {
    pub vertices: Vec<V>,
    pub indices: Vec<usize>,
}

pub trait VertexBufferExt {
    fn to_bytes(&self, bytes: &mut Vec<u8>);
}

impl<V: VertexBufferExt> VertexBufferBuilder<V> {
    pub fn new(vertices: Vec<V>) -> Self {
        Self {
            vertices,
            indices: vec![],
        }
    }

    pub fn with_indices(mut self, indices: Vec<usize>) -> Self {
        self.indices = indices;
        self
    }

    pub fn append(&mut self, mut vertices: Vec<V>, mut indices: Vec<usize>) {
        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        // Emit the data into the buffer.
        for vertex in self.vertices.iter() {
            vertex.to_bytes(&mut bytes);
        }

        bytes
    }

    pub fn build(self, device: &wgpu::Device) -> VertexBuffer<V> {
        let bytes = self.to_bytes();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &bytes,
            usage: wgpu::BufferUsages::COPY_SRC,
            label: Some("vertex Build Buffer"),
        });

        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::COPY_SRC,
            label: Some("indices Build Buffer"),
        });

        VertexBuffer {
            vertex_buffer,
            indice_buffer,
            vertices: self.vertices,
            indices: self.indices,
            bytes,
        }
    }
}

#[derive(Debug)]
pub struct VertexBuffer<V> {
    vertex_buffer: wgpu::Buffer,
    indice_buffer: wgpu::Buffer,
    vertices: Vec<V>,
    indices: Vec<usize>,
    bytes: Vec<u8>,
}

impl<V> VertexBuffer<V> {
    pub fn vertices(&self) -> &[V] {
        &self.vertices
    }

    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn indice_buffer(&self) -> &wgpu::Buffer {
        &self.indice_buffer
    }
}
