use crate::graphics::{LayoutStorage, MapLayout, MapTextures};

pub struct MapGroup {
    pub bind_group: wgpu::BindGroup,
}

impl MapGroup {
    pub fn from_maps(
        device: &wgpu::Device,
        layout_storage: &mut LayoutStorage,
        texture: &MapTextures,
    ) -> Self {
        let entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(texture.view()),
        }];

        let layout = layout_storage.create_layout(device, MapLayout);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some("Map Texture Bind Group"),
        });

        Self { bind_group }
    }
}
