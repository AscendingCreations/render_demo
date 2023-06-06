use crate::WorldBounds;
use std::ops::Range;

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
