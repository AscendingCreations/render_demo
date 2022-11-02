use crate::graphics::{Layout, LayoutStorage, Renderer};
use bytemuck::{Pod, Zeroable};
use crevice::std140::{AsStd140, Std140};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct TextColoredLayout;

impl Layout for TextColoredLayout {
    fn create_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text_colored_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

#[derive(AsStd140)]
pub struct TextColoredUniform {
    //which texture array to use for the glyph.
    colored: u32,
}

pub struct TextColoredGroup {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl TextColoredGroup {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn new(
        renderer: &Renderer,
        layout_storage: &mut LayoutStorage,
    ) -> Self {
        let colored_info = TextColoredUniform { colored: 0 };

        // Create the uniform buffer.
        let buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("text colored buffer"),
                contents: colored_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout =
            layout_storage.create_layout(renderer.device(), TextColoredLayout);

        // Create the bind group.
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("text_colored_bind_group"),
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });

        Self { buffer, bind_group }
    }

    pub fn update(&mut self, renderer: &Renderer, colored: bool) {
        let colored_info = TextColoredUniform {
            colored: colored as u32,
        };

        renderer.queue().write_buffer(
            &self.buffer,
            0,
            colored_info.as_std140().as_bytes(),
        );
    }
}
