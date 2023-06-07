use crate::{GpuDevice, OrderedIndex, WorldBounds};
use std::{cmp::Ordering, marker::PhantomData, ops::Range};
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct BufferStore {
    pub store: Vec<u8>,
    pub indexs: Vec<u8>,
    //bounds and height to reverse the clipping for Windows Scissor routine.
    pub bounds: Option<WorldBounds>,
    pub changed: bool,
    pub store_pos: Range<usize>,
    pub index_pos: Range<usize>,
}

pub struct BufferPass<'a> {
    pub vertex_buffer: &'a wgpu::Buffer,
    pub index_buffer: &'a wgpu::Buffer,
}

pub trait AsBufferPass<'a> {
    fn as_buffer_pass(&'a self) -> BufferPass<'a>;
}

#[derive(Default)]
pub struct BufferData {
    pub vertexs: Vec<u8>,
    pub indexs: Vec<u8>,
}

//need this to render each mesh as an instance.
pub struct BufferDetails {
    pub order_index: OrderedIndex,
    pub vertex_count: usize,
    pub index_count: usize,
}

impl PartialOrd for BufferDetails {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BufferDetails {
    fn eq(&self, other: &Self) -> bool {
        self.order_index == other.order_index
    }
}

impl Eq for BufferDetails {}

impl Ord for BufferDetails {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order_index.cmp(&other.order_index)
    }
}

pub struct Buffer<K: BufferLayout> {
    pub buffer: wgpu::Buffer,
    pub count: usize,
    pub len: usize,
    pub max: usize,
    phantom_data: PhantomData<K>,
}

impl<K: BufferLayout> Buffer<K> {
    pub fn new(
        gpu_device: &GpuDevice,
        contents: &[u8],
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        Self {
            buffer: gpu_device.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label,
                    contents,
                    usage,
                },
            ),
            count: 0,
            len: 0,
            max: contents.len(),
            phantom_data: PhantomData,
        }
    }

    pub fn write(&self, device: &GpuDevice, data: &[u8], pos: u64) {
        device.queue.write_buffer(&self.buffer, pos, data);
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn buffer_slice(&self, range: Range<u64>) -> wgpu::BufferSlice {
        self.buffer.slice(range)
    }
}

pub trait BufferLayout {
    fn is_bounded() -> bool;
    ///WGPU's Shader Attributes
    fn attributes() -> Vec<wgpu::VertexAttribute>;

    ///Default Buffer set to a large size.
    fn default_buffer() -> BufferData;

    ///The size in bytes the vertex is
    fn stride() -> usize;

    /// Creates a Buffer at a capacity
    /// Capacity is a count of objects.
    fn with_capacity(
        vertex_capacity: usize,
        index_capacity: usize,
    ) -> BufferData;
}
