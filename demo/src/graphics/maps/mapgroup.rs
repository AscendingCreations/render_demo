use crate::graphics::{Atlas, LayoutStorage, MapLayout};

pub struct MapGroup {
    pub bind_group: wgpu::BindGroup,
}

impl MapGroup {
    pub fn from_map(device: &wgpu::Device, layout_storage: &mut LayoutStorage, map: &Map) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&map.texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ];

        let layout = layout_storage.create_layout(device, MapLayout);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some("Texture Bind Group"),
        });

        Self { bind_group }
    }
}
