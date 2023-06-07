use crate::{GpuDevice, WorldBounds};
use std::ops::Range;
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct BufferStore {
    pub store: Vec<u8>,
    pub indexs: Vec<u8>,
    //bounds and height to reverse the clipping for Windows Scissor routine.
    pub bounds: Option<WorldBounds>,
    pub changed: bool,
    pub pos: Range<usize>,
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

pub struct Buffer {
    pub buffer: wgpu::Buffer,
    pub count: usize,
    pub len: usize,
    pub max: usize,
}

impl Buffer {
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
        }
    }

    pub fn write(&self, device: &GpuDevice, data: &[u8], pos: u64) {
        device.queue.write_buffer(&self.buffer, pos, &data);
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn buffer_slice(&self, range: Range<u64>) -> wgpu::BufferSlice {
        self.buffer.slice(range)
    }
}
