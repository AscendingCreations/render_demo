use crate::{AsBufferPass, BufferData, BufferPass, GpuDevice};
use std::{marker::PhantomData, ops::Range};
use wgpu::util::DeviceExt;

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
    pub vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
    vertex_max: usize,
    pub index_buffer: wgpu::Buffer,
    index_count: usize,
    index_max: usize,
    // Ghost Data that doesnt Actually exist. Used to set the Generic to a trait.
    // without needing to set it to a variable that is loaded.
    phantom_data: PhantomData<K>,
}

impl<'a, K: BufferLayout> AsBufferPass<'a> for GpuBuffer<K> {
    fn as_buffer_pass(&'a self) -> BufferPass<'a> {
        BufferPass {
            vertex_buffer: &self.vertex_buffer,
            index_buffer: &self.index_buffer,
        }
    }
}

impl<K: BufferLayout> GpuBuffer<K> {
    /// Used to create GpuBuffer from a (Vertex:Vec<u8>, Indices:Vec<u8>).
    pub fn create_buffer(gpu_device: &GpuDevice, buffers: &BufferData) -> Self {
        GpuBuffer {
            vertex_buffer: gpu_device.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &buffers.vertexs,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST,
                },
            ),
            vertex_count: 0,
            vertex_max: buffers.vertexs.len(),
            index_buffer: gpu_device.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: &buffers.indexs,
                    usage: wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST,
                },
            ), // set to 0 as we set this as we add sprites.
            index_count: 0, //(buffers.indices.len() / K::index_stride()),
            index_max: buffers.indexs.len(),
            phantom_data: PhantomData,
        }
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(&mut self, gpu_device: &GpuDevice, capacity: usize) {
        let buffers = K::with_capacity(capacity);

        self.vertex_buffer = gpu_device.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &buffers.vertexs,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
            },
        );
        self.vertex_max = buffers.vertexs.len();
        self.index_buffer = gpu_device.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: &buffers.indexs,
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        self.index_max = buffers.indexs.len();
    }

    /// Returns the index_count.
    pub fn index_count(&self) -> usize {
        self.index_count
    }

    /// Returns the index maximum size.
    pub fn index_max(&self) -> usize {
        self.index_max
    }

    /// Returns wgpu::BufferSlice of indices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..index_count.
    pub fn indices(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..(self.index_count) as u64
        };

        self.index_buffer.slice(range)
    }

    /// creates a new pre initlized VertexBuffer with a default size.
    /// default size is based on the initial BufferPass::vertices length.
    pub fn new(renderer: &GpuDevice) -> Self {
        Self::create_buffer(renderer, &K::default_buffer())
    }

    /// Set the Index based on how many Vertex's Exist
    pub fn set_index_count(&mut self, count: usize) {
        self.index_count = count;
    }

    /// Sets the index_buffer from byte array of indices.
    /// Also sets index_count to array length / index_stride.
    pub fn set_indices_from(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        let size = bytes.len();

        if size >= self.index_max {
            return;
        }

        self.index_count = size;
        queue.write_buffer(&self.index_buffer, 0, bytes);
    }

    /// Sets the vertex_buffer from byte array of vertices.
    /// Sets the vertex_count to array length / vertex_stride.
    /// Sets the index_count to vertex_count / index_offset.
    /// Will resize both vertex_buffer and index_buffer if bytes length is larger than vertex_max.
    pub fn set_vertices_from(
        &mut self,
        gpu_device: &GpuDevice,
        queue: &wgpu::Queue,
        bytes: &[u8],
    ) {
        let size = bytes.len();

        if size > self.vertex_max {
            self.resize(gpu_device, size / K::vertex_stride());
        }

        self.vertex_count = size / K::vertex_stride();
        self.index_count = self.vertex_count;

        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    /// Sets both buffers from another 'BufferPass'
    pub fn set_buffers_from(
        &mut self,
        gpu_device: &GpuDevice,
        buffers: BufferData,
    ) {
        let vertex_size = buffers.vertexs.len();
        let index_size = buffers.indexs.len();

        if vertex_size > self.vertex_max {
            self.resize(gpu_device, vertex_size / K::vertex_stride());
        }

        if index_size > self.index_max {
            return;
        }

        self.vertex_count = vertex_size / K::vertex_stride();
        self.index_count = self.vertex_count;
        gpu_device
            .queue
            .write_buffer(&self.vertex_buffer, 0, &buffers.vertexs);
        gpu_device
            .queue
            .write_buffer(&self.index_buffer, 0, &buffers.indexs);
    }

    /// Returns the Vertex elements count.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Returns vertex_buffer's max size in bytes.
    pub fn vertex_max(&self) -> usize {
        self.vertex_max
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
            0..self.vertex_count as u64
        };

        self.vertex_buffer.slice(range)
    }

    /// Creates a GpuBuffer based on capacity.
    /// Capacity is the amount of objects to initialize for.
    pub fn with_capacity(gpu_device: &GpuDevice, capacity: usize) -> Self {
        Self::create_buffer(gpu_device, &K::with_capacity(capacity))
    }
}
