use crate::graphics::SpriteVertex;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

pub struct BufferPass {
    pub vertices: Vec<u8>,
    pub indices: Vec<u8>,
}

pub trait BufferLayout {
    fn stride() -> u64;
    fn initial_buffer() -> BufferPass;
    fn attributes() -> Vec<wgpu::VertexAttribute>;
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
    /// creates a new pre initlized VertexBuffer with a Static size.
    /// static size is based on the initial BufferPass::vertices length.
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers = K::initial_buffer();
        let buffer_stride = K::stride();
        let buffer_max = buffers.vertices.len() as u64;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &buffers.vertices,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            label: Some("Vertex Buffer"),
        });

        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &buffers.indices,
            usage: wgpu::BufferUsages::INDEX,
            label: Some("Indices Buffer"),
        });

        VertexBuffer {
            vertex_buffer,
            indice_buffer,
            indice_count: (buffers.indices.len() / 4) as u64,
            vertex_count: 0, // set to 0 as we set this as we add sprites.
            buffer_max,
            buffer_stride,
            phantom_data: PhantomData,
        }
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

    /// Gets the indice count.
    pub fn indice_count(&self) -> u64 {
        self.indice_count
    }

    /// Gets Vertex buffers max size in bytes.
    pub fn buffer_max(&self) -> u64 {
        self.buffer_max
    }

    /// Gets Vertex buffers struct stride.
    pub fn buffer_stride(&self) -> u64 {
        self.buffer_stride
    }
}
