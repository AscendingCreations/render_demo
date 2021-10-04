use crate::graphics::SpriteVertex;
use std::iter;
use wgpu::util::DeviceExt;

pub struct SpriteBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub indice_buffer: wgpu::Buffer,
    pub vertex_count: u64, //Count of indices per Vertex layout within Buffer. used for rendering
    pub indice_count: u64,
}

impl SpriteBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_arr: Vec<SpriteVertex> = iter::repeat(SpriteVertex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0, 0.0],
            color: [0, 0, 0, 100],
        })
        .take(40_000)
        .collect();

        let indices = (0..10_000)
            .map(|x| vec![x, x + 1, x + 2, x, x + 2, x + 3])
            .flatten()
            .collect::<Vec<u32>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &bytemuck::cast_slice(&vertex_arr).to_vec(),
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
            indice_count: indices.len() as u64,
            vertex_count: 0, //set to 0 as we set this as we add sprites.
        }
    }

    pub fn set_buffer(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        if (bytes.len() / 40) as u64 >= 40_000 {
            return; //so I dont accidently go over Will change this later to be adaptable for now static
        }

        self.vertex_count = (bytes.len() / 40) as u64;
        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    pub fn set_indice_count(&mut self, count: u64) {
        self.indice_count = count;
        //we will just set the Count since vertex is static we only
        //need to handle how many indices to allow for the vertices.
    }
}
