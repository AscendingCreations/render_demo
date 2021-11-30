pub(crate) use crate::graphics::SpriteVertex;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

pub struct BufferPass {
    pub vertices: Vec<u8>,
    pub indices: Vec<u8>,
}

pub trait BufferLayout {
    fn attributes() -> Vec<wgpu::VertexAttribute>;
    fn default_buffer() -> BufferPass;
    fn with_capacity(capacity: usize) -> BufferPass;
    fn stride() -> u64;
}

pub struct VertexBuffer<K: BufferLayout> {
    pub vertex_buffer: wgpu::Buffer,
    pub indice_buffer: wgpu::Buffer,
    vertex_count: u64,
    indice_count: u64,
    buffer_max: u64,
    buffer_stride: u64,
    phantom_data: PhantomData<K>,
}

impl<K: BufferLayout> VertexBuffer<K> {
    /// Gets Vertex buffers max size in bytes.
    pub fn buffer_max(&self) -> u64 {
        self.buffer_max
    }

    /// Gets Vertex buffers struct stride.
    pub fn buffer_stride(&self) -> u64 {
        self.buffer_stride
    }

    fn create_buffer(device: &wgpu::Device, buffers: BufferPass) -> Self {
        VertexBuffer {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &buffers.vertices,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            }),
            indice_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Indices Buffer"),
                contents: &buffers.indices,
                usage: wgpu::BufferUsages::INDEX,
            }),
            vertex_count: 0,
            indice_count: (buffers.indices.len() / 4) as u64, // set to 0 as we set this as we add sprites.
            buffer_max: buffers.vertices.len() as u64,
            buffer_stride: K::stride(),
            phantom_data: PhantomData,
        }
    }

    /// Gets the indice count.
    pub fn indice_count(&self) -> u64 {
        self.indice_count
    }
    /// creates a new pre initlized VertexBuffer with a default size.
    /// default size is based on the initial BufferPass::vertices length.
    pub fn new(device: &wgpu::Device) -> Self {
        Self::create_buffer(device, K::default_buffer())
    }

    /// Set the New buffer array to the VertexBuffer.
    /// Sets the vertex_count based on array length / struct stride.
    pub fn set_buffer(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        let size = bytes.len() as u64;

        if size >= self.buffer_max {
            return;
        }

        self.vertex_count = size / self.buffer_stride;
        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    /// Set the Indices based on how many Vertex's Exist.
    pub fn set_indice_count(&mut self, count: u64) {
        self.indice_count = count;
    }

    /// Gets the Vertex elements count.
    pub fn vertex_count(&self) -> u64 {
        self.vertex_count
    }

    /// creates a new pre initlized VertexBuffer with a entity count.
    /// size created BufferPass::vertices length.
    pub fn with_capacity(device: &wgpu::Device, capacity: usize) -> Self {
        Self::create_buffer(device, K::with_capacity(capacity))
    }
}
