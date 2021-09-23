use crate::graphics::{Vertex, VertexBufferBuilder};
use std::iter;
use wgpu::util::DeviceExt;

pub struct SpriteBuffer {
    vertex_buffer: wgpu::Buffer,
    vertex_size: wgpu::BufferAddress,
    index_buffer: wgpu::Buffer,
    index_size: wgpu::BufferAddress,
    vertex_count: usize,
    num_indices: usize,
}

pub fn size_of_slice<T: Sized>(slice: &[T]) -> usize {
    std::mem::size_of::<T>() * slice.len()
}

impl SpriteBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_arr: Vec<Vertex> = iter::repeat(Vertex {
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
            .build();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &buffer.bytes(),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            label: Some("vertex Buffer"),
        });

        let num_indices = buffer.indices().len();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&buffer.indices()),
            usage: wgpu::BufferUsages::INDEX,
            label: Some("indices Buffer"),
        });

        let vertex_size = size_of_slice(&buffer.bytes()) as wgpu::BufferAddress;
        let index_size = size_of_slice(&buffer.indices()) as wgpu::BufferAddress;

        SpriteBuffer {
            vertex_buffer,
            vertex_size,
            index_buffer,
            index_size,
            num_indices,
            vertex_count: 0,
        }
    }
}
