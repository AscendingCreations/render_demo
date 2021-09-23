#[derive(Debug)]
pub struct VertexBufferBuilder<V: VertexBufferExt> {
    vertices: Vec<V>,
    indices: Vec<usize>,
}

pub trait VertexBufferExt {
    fn to_bytes(self, bytes: &mut Vec<u8>);
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

    pub fn append(&mut self, vertices: Vec<V>, indices: Vec<usize>) {
        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        // Emit the data into the buffer.
        for vertex in &self.vertices {}

        bytes
    }

    pub fn build(mut self) -> VertexBuffer<V> {
        let bytes = self.to_bytes();

        VertexBuffer {
            vertices: self.vertices,
            indices: self.indices,
            bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VertexBuffer<V> {
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
}
