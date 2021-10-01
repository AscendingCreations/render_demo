use crate::graphics::{Atlas, LayoutStorage, TextureLayout};

pub struct TextureGroup {
    pub bind_group: wgpu::BindGroup,
}

impl TextureGroup {
    pub fn from_atlas(
        device: &wgpu::Device,
        layout_storage: &mut LayoutStorage,
        atlas: &Atlas,
    ) -> Self {
        let entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&atlas.texture_view),
        }];

        let layout = layout_storage.create_layout(device, TextureLayout);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some("Texture Bind Group"),
        });

        Self { bind_group }
    }
}
