use crate::graphics::{SpriteVertex, VertexBufferBuilder};
use std::iter;
use wgpu::util::DeviceExt;

pub struct SpriteBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_size: wgpu::BufferAddress,
    pub indice_buffer: wgpu::Buffer,
    pub indice_size: wgpu::BufferAddress,
    pub vertex_count: usize,
    pub num_indices: usize,
}

pub fn size_of_slice<T: Sized>(slice: &[T]) -> usize {
    std::mem::size_of::<T>() * slice.len()
}

impl SpriteBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_arr: Vec<SpriteVertex> = iter::repeat(SpriteVertex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0, 1.0],
            color: [0.0, 0.0, 0.0, 0.0],
        })
        .take(40_000)
        .collect();

        let indices = (0..10_000)
            .map(|x| vec![x, x + 1, x + 2, x + 1, x + 2, x + 3])
            .flatten()
            .collect();

        let buffer = VertexBufferBuilder::new(vertex_arr)
            .with_indices(indices)
            .build(device);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: buffer.bytes(),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            label: Some("vertex Buffer"),
        });

        let num_indices = buffer.indices().len();
        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(buffer.indices()),
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            label: Some("indices Buffer"),
        });

        let vertex_size = size_of_slice(buffer.bytes()) as wgpu::BufferAddress;
        let indice_size = size_of_slice(buffer.indices()) as wgpu::BufferAddress;

        SpriteBuffer {
            vertex_buffer,
            vertex_size,
            indice_buffer,
            indice_size,
            num_indices,
            vertex_count: 0,
        }
    }

    pub fn copy_to_vertex(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        other: (&wgpu::Buffer, wgpu::BufferAddress),
        count: usize,
    ) {
        self.vertex_count = count;
        encoder.copy_buffer_to_buffer(&self.vertex_buffer, 0, other.0, 0, other.1)
    }

    pub fn copy_to_indice(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        other: (&wgpu::Buffer, wgpu::BufferAddress),
        count: usize,
    ) {
        self.num_indices = count;
        encoder.copy_buffer_to_buffer(&self.indice_buffer, 0, other.0, 0, other.1)
    }
}
