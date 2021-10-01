use crate::graphics::MapVertex;
use std::iter;
use wgpu::util::DeviceExt;

pub struct MapBuffer {
    //everything is setup for a 9x9 map layout
    pub vertex_buffer: wgpu::Buffer,
    pub indice_buffer: wgpu::Buffer,
    pub vertex_count: u64, //Count of indices per Vertex layout within Buffer. used for rendering
    pub indice_count: u64,
}

impl MapBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_arr: Vec<MapVertex> = iter::repeat(MapVertex {
            position: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0, 1.0],
        })
        .take(1_568)
        .collect();

        let indices = (0..392)
            .map(|x| vec![x, x + 1, x + 2, x, x + 2, x + 3])
            .flatten()
            .collect::<Vec<u32>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: &bytemuck::cast_slice(&vertex_arr).to_vec(),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            label: Some("Map Vertex Buffer"),
        });

        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
            label: Some("Map Indices Buffer"),
        });

        MapBuffer {
            vertex_buffer,
            indice_buffer,
            indice_count: indices.len() as u64,
            vertex_count: 0, //set to 0 as we set this as we add sprites.
        }
    }

    pub fn set_buffer(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        if ((bytes.len() / 6) / 4) as u64 >= 1_568 {
            return; //so I dont accidently go over Will change this later to be adaptable for now static
        }

        self.vertex_count = ((bytes.len() / 6) / 4) as u64;
        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    pub fn set_indice_count(&mut self, count: u64) {
        self.indice_count = count;
    }
}
