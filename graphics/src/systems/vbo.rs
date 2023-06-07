use crate::{
    AsBufferPass, Buffer, BufferData, BufferPass, GpuDevice, OrderedIndex,
    WorldBounds,
};
use std::{marker::PhantomData, ops::Range};

//This Holds onto all the Vertexs Compressed into a byte array.
//This is Used for objects that need more advanced VBO/IBO other wise use the Instance buffers.

pub trait BufferLayout {
    fn is_bounded() -> bool;
    ///WGPU's Shader Attributes
    fn attributes() -> Vec<wgpu::VertexAttribute>;

    ///Default Buffer set to a large size.
    fn default_buffer() -> BufferData;

    ///The size in bytes the vertex is
    fn vertex_stride() -> usize;

    /// Creates a Buffer at a capacity
    /// Capacity is a count of objects.
    fn with_capacity(capacity: usize) -> BufferData;
}

pub struct GpuBuffer<K: BufferLayout> {
    pub buffers: Vec<OrderedIndex>,
    pub vertex_buffer: Buffer,
    pub vertex_needed: usize,
    pub index_buffer: Buffer,
    pub index_needed: usize,
    pub bounds: Vec<Option<WorldBounds>>,
    // Ghost Data that doesnt Actually exist. Used to set the Generic to a trait.
    // without needing to set it to a variable that is loaded.
    phantom_data: PhantomData<K>,
}

impl<'a, K: BufferLayout> AsBufferPass<'a> for GpuBuffer<K> {
    fn as_buffer_pass(&'a self) -> BufferPass<'a> {
        BufferPass {
            vertex_buffer: &self.vertex_buffer.buffer,
            index_buffer: &self.index_buffer.buffer,
        }
    }
}

impl<K: BufferLayout> GpuBuffer<K> {
    /// Used to create GpuBuffer from a (Vertex:Vec<u8>, Indices:Vec<u8>).
    pub fn create_buffer(gpu_device: &GpuDevice, buffers: &BufferData) -> Self {
        GpuBuffer {
            buffers: Vec::with_capacity(256),
            vertex_buffer: Buffer::new(
                gpu_device,
                &buffers.vertexs,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some("Vertex Buffer"),
            ),
            vertex_needed: 0,
            index_buffer: Buffer::new(
                gpu_device,
                &buffers.indexs,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                Some("Index Buffer"),
            ),
            index_needed: 0,
            bounds: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(&mut self, gpu_device: &GpuDevice, capacity: usize) {
        let buffers = K::with_capacity(capacity);

        self.vertex_buffer = Buffer::new(
            gpu_device,
            &buffers.vertexs,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("Vertex Buffer"),
        );

        self.index_buffer = Buffer::new(
            gpu_device,
            &buffers.indexs,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            Some("Index Buffer"),
        )
    }

    /// Returns the index_count.
    pub fn index_count(&self) -> usize {
        self.index_buffer.count
    }

    /// Returns the index maximum size.
    pub fn index_max(&self) -> usize {
        self.index_buffer.max
    }

    /// Returns wgpu::BufferSlice of indices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..index_count.
    pub fn indices(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..(self.index_buffer.count) as u64
        };

        self.index_buffer.buffer_slice(range)
    }

    /// creates a new pre initlized VertexBuffer with a default size.
    /// default size is based on the initial BufferPass::vertices length.
    pub fn new(device: &GpuDevice) -> Self {
        Self::create_buffer(device, &K::default_buffer())
    }

    /// Set the Index based on how many Vertex's Exist
    pub fn set_index_count(&mut self, count: usize) {
        self.index_buffer.count = count;
    }

    /// Sets the index_buffer from byte array of indices.
    /// Also sets index_count to array length / index_stride.
    pub fn set_indices_from(&mut self, device: &GpuDevice, bytes: &[u8]) {
        let size = bytes.len();

        if size >= self.index_buffer.max {
            return;
        }

        self.index_buffer.count = size;
        self.index_buffer.write(device, bytes, 0);
    }

    /// Sets the vertex_buffer from byte array of vertices.
    /// Sets the vertex_count to array length / vertex_stride.
    /// Sets the index_count to vertex_count / index_offset.
    /// Will resize both vertex_buffer and index_buffer if bytes length is larger than vertex_max.
    pub fn set_vertices_from(
        &mut self,
        gpu_device: &GpuDevice,
        bytes: &[u8],
        bounds: &[Option<WorldBounds>],
    ) {
        let size = bytes.len();

        if size > self.vertex_buffer.max {
            self.resize(gpu_device, size / K::vertex_stride());
        }

        self.vertex_buffer.count = size / K::vertex_stride();
        self.index_buffer.count = self.vertex_buffer.count;
        self.bounds = bounds.to_vec();

        self.vertex_buffer.write(gpu_device, bytes, 0);
    }

    /// Sets both buffers from another 'BufferPass'
    pub fn set_buffers_from(
        &mut self,
        gpu_device: &GpuDevice,
        buffers: BufferData,
        bounds: &[Option<WorldBounds>],
    ) {
        let vertex_size = buffers.vertexs.len();
        let index_size = buffers.indexs.len();

        if vertex_size > self.vertex_buffer.max {
            self.resize(gpu_device, vertex_size / K::vertex_stride());
        }

        if index_size > self.index_buffer.max {
            return;
        }

        self.vertex_buffer.count = vertex_size / K::vertex_stride();
        self.index_buffer.count = self.vertex_buffer.count;
        self.bounds = bounds.to_vec();

        gpu_device.queue.write_buffer(
            &self.vertex_buffer.buffer,
            0,
            &buffers.vertexs,
        );
        gpu_device.queue.write_buffer(
            &self.index_buffer.buffer,
            0,
            &buffers.indexs,
        );
    }

    /// Returns the Vertex elements count.
    pub fn vertex_count(&self) -> usize {
        self.vertex_buffer.count
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_buffer.count == 0
    }

    /// Returns vertex_buffer's max size in bytes.
    pub fn vertex_max(&self) -> usize {
        self.vertex_buffer.max
    }

    /// Returns vertex_buffer's vertex_stride.
    pub fn vertex_stride(&self) -> usize {
        K::vertex_stride()
    }

    /// Returns wgpu::BufferSlice of vertices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..vertex_count.
    pub fn vertices(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..self.vertex_buffer.count as u64
        };

        self.vertex_buffer.buffer_slice(range)
    }

    /// Creates a GpuBuffer based on capacity.
    /// Capacity is the amount of objects to initialize for.
    pub fn with_capacity(gpu_device: &GpuDevice, capacity: usize) -> Self {
        Self::create_buffer(gpu_device, &K::with_capacity(capacity))
    }
}
