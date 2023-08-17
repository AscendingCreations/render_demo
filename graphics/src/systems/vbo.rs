use crate::{
    AsBufferPass, Buffer, BufferData, BufferLayout, BufferPass, GpuDevice,
    GpuRenderer, OrderedIndex,
};
use std::ops::Range;

//This Holds onto all the Vertexs Compressed into a byte array.
//This is Used for objects that need more advanced VBO/IBO other wise use the Instance buffers.

pub struct IndexDetails {
    pub count: u32,
    pub max: u32,
}

pub struct GpuBuffer<K: BufferLayout> {
    unprocessed: Vec<OrderedIndex>,
    pub buffers: Vec<IndexDetails>,
    pub vertex_buffer: Buffer<K>,
    vertex_needed: usize,
    pub index_buffer: Buffer<K>,
    index_needed: usize,
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
            unprocessed: Vec::with_capacity(256),
            buffers: Vec::new(),
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
        }
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        mut index: OrderedIndex,
    ) {
        if let Some(store) = renderer.get_buffer(&index.index) {
            self.vertex_needed += store.store.len();
            self.index_needed += store.indexs.len();

            index.index_count = store.indexs.len() as u32 / 4;

            self.unprocessed.push(index);
        }
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        let (mut changed, mut vertex_pos, mut index_pos) = (false, 0, 0);

        if self.vertex_needed > self.vertex_buffer.max
            || self.index_needed > self.index_buffer.max
        {
            self.resize(
                renderer.gpu_device(),
                self.vertex_needed / K::stride(),
                self.index_needed,
            );
            changed = true;
        }

        self.vertex_buffer.count = self.vertex_needed / K::stride();
        self.vertex_buffer.len = self.vertex_needed;

        self.unprocessed.sort();
        self.buffers.clear();

        for buf in &self.unprocessed {
            let mut write_vertex = false;
            let mut write_index = false;
            let old_vertex_pos = vertex_pos as u64;
            let old_index_pos = index_pos as u64;

            if let Some(store) = renderer.get_buffer_mut(&buf.index) {
                let vertex_range = vertex_pos..vertex_pos + store.store.len();
                let index_range = index_pos..index_pos + store.indexs.len();

                if store.store_pos != vertex_range || changed || store.changed {
                    store.store_pos = vertex_range;
                    write_vertex = true
                }

                if store.index_pos != index_range || changed || store.changed {
                    store.index_pos = index_range;
                    write_index = true
                }

                if write_index || write_vertex {
                    store.changed = false;
                }

                vertex_pos += store.store.len();
                index_pos += store.indexs.len();
            }

            if write_vertex {
                if let Some(store) = renderer.get_buffer(&buf.index) {
                    self.vertex_buffer.write(
                        &renderer.device,
                        &store.store,
                        old_vertex_pos,
                    );
                }
            }

            if write_index {
                if let Some(store) = renderer.get_buffer(&buf.index) {
                    self.index_buffer.write(
                        &renderer.device,
                        &store.indexs,
                        old_index_pos,
                    );
                }
            }

            self.buffers.push(IndexDetails {
                count: buf.index_count,
                max: buf.index_max,
            });
        }

        self.unprocessed.clear();
        self.vertex_needed = 0;
        self.index_needed = 0;
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(
        &mut self,
        gpu_device: &GpuDevice,
        vertex_capacity: usize,
        index_capacity: usize,
    ) {
        let buffers = K::with_capacity(vertex_capacity, index_capacity);

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
        K::stride()
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
    /// Capacity * 2 == the reserved space for the indices.
    pub fn with_capacity(gpu_device: &GpuDevice, capacity: usize) -> Self {
        Self::create_buffer(
            gpu_device,
            &K::with_capacity(capacity, capacity * 2),
        )
    }
}
