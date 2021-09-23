use crate::graphics::Layout;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct TextureLayout;

impl Layout for TextureLayout {
    fn create_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    comparison: false,
                    filtering: true,
                },
                count: None,
            },
        ];

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("texture_bind_group_layout"),
        })
    }
}
