use crate::{GpuDevice, Layout};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct MapLayout;

impl Layout for MapLayout {
    fn create_layout(&self, gpu_device: &GpuDevice) -> wgpu::BindGroupLayout {
        let entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    sample_type: wgpu::TextureSampleType::Uint,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(
                    wgpu::SamplerBindingType::Filtering,
                ),
                count: None,
            },
        ];

        gpu_device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Map_texture_bind_group_layout"),
                entries: &entries,
            },
        )
    }
}
