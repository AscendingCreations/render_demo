pub(crate) use super::{Layout, LayoutStorage, Renderer};
use bytemuck::{Pod, Zeroable};
use crevice::std140::{AsStd140, Std140};
use std::{collections::HashSet, iter, mem::size_of, num::NonZeroU32, slice};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct ScreenLayout;

impl Layout for ScreenLayout {
    fn create_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen_bind_group_layout"),
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

// This is more for Rendering GUI as we dont need a Camera for anything GUI based.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, AsStd140)]
pub struct ScreenUniform {
    //width of the view area
    pub width: u32,
    //height of the view area
    pub height: u32,
}

pub struct ScreenGroup {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pub screen_size: ScreenUniform,
}

impl ScreenGroup {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn new(
        renderer: &Renderer,
        layout_storage: &mut LayoutStorage,
        screen_size: ScreenUniform,
    ) -> Self {
        // Create the uniform buffer.
        let buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("screen size buffer"),
                contents: screen_size.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout =
            layout_storage.create_layout(renderer.device(), ScreenLayout);

        // Create the bind group.
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Screen_size_bind_group"),
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });

        Self {
            buffer,
            bind_group,
            screen_size,
        }
    }

    pub fn update(&mut self, renderer: &Renderer, screen_size: ScreenUniform) {
        if self.screen_size != screen_size {
            self.screen_size = screen_size;
            renderer.queue().write_buffer(
                &self.buffer,
                0,
                self.screen_size.as_std140().as_bytes(),
            );
        }
    }
}
