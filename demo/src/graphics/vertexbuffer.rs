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
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            label: Some("vertex Build Buffer"),
        });

        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            label: Some("indices Build Buffer"),
        });

        let vertex_size = super::sprites::size_of_slice(&bytes) as wgpu::BufferAddress;
        let indice_size = super::sprites::size_of_slice(&self.indices) as wgpu::BufferAddress;

        VertexBuffer {
            vertex_buffer,
            indice_buffer,
            vertex_size,
            indice_size,
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
    vertex_size: wgpu::BufferAddress,
    indice_size: wgpu::BufferAddress,
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

    pub fn vertex_buffer(&self) -> (&wgpu::Buffer, wgpu::BufferAddress) {
        (&self.vertex_buffer, self.vertex_size)
    }

    pub fn indice_buffer(&self) -> (&wgpu::Buffer, wgpu::BufferAddress) {
        (&self.indice_buffer, self.indice_size)
    }
}
