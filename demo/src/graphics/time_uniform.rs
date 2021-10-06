use bytemuck::{Pod, Zeroable};
use crevice::std140::{AsStd140, Std140};
use wgpu::util::DeviceExt;

use super::{Layout, LayoutStorage, Renderer};

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct TimeLayout;

impl Layout for TimeLayout {
    fn create_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("time_bind_group_layout"),
        })
    }
}

#[derive(AsStd140)]
pub struct TimeUniform {
    //seconds since the start of the program. given by the FrameTime
    seconds: f32,
}

pub struct TimeGroup {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl TimeGroup {
    pub fn new(renderer: &Renderer, layout_storage: &mut LayoutStorage) -> Self {
        let time_info = TimeUniform { seconds: 0.0 };

        // Create the uniform buffer.
        let buffer = renderer
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("time buffer"),
                contents: time_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create the bind group layout for the camera.
        let layout = layout_storage.create_layout(renderer.device(), TimeLayout);

        // Create the bind group.
        let bind_group = renderer
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("time_bind_group"),
            });

        Self { buffer, bind_group }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update(&mut self, renderer: &Renderer, seconds: f32) {
        let time_info = TimeUniform { seconds };

        renderer
            .queue()
            .write_buffer(&self.buffer, 0, time_info.as_std140().as_bytes());
    }
}
