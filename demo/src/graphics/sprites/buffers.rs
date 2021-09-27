use crate::graphics::SpriteVertex;
use std::iter;
use wgpu::util::DeviceExt;

pub struct SpriteBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub indice_buffer: wgpu::Buffer,
    pub vertex_size: u64, //Max current size of buffer.
    pub indice_size: u64,
    pub vertex_count: u64, //Count of indices per Vertex layout within Buffer. used for rendering
    pub indice_count: u64,
}

pub fn size_of_slice<T: Sized>(slice: &[T]) -> u64 {
    (std::mem::size_of::<T>() * slice.len()) as u64
}

impl SpriteBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut bytes = vec![];

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
            .collect::<Vec<u32>>();

        for vertex in vertex_arr.iter() {
            vertex.to_bytes(&mut bytes);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &bytes,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            label: Some("vertex Buffer"),
        });

        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
            label: Some("indices Buffer"),
        });

        SpriteBuffer {
            vertex_buffer,
            indice_buffer,
            vertex_size: size_of_slice(&bytes),
            indice_size: size_of_slice(&indices),
            indice_count: indices.len() as u64,
            vertex_count: 0, //set to 0 as we set this as we add sprites.
        }
    }

    pub fn set_buffer(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        if bytes.len() >= 40_000 {
            return; //so I dont accidently go over Will change this later to be adaptable for now static
        }

        self.vertex_count = bytes.len() as u64;
        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    pub fn set_indice_count(&mut self, count: u64) {
        self.indice_count = count;
        //we will just set the Count since vertex is static we only
        //need to handle how many indices to allow for the vertices.
    }
}
