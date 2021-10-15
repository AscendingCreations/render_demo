use crate::graphics::{Atlas, Layout, LayoutStorage, TextureLayout};

pub struct TextureGroup {
    pub bind_group: wgpu::BindGroup,
}

impl TextureGroup {
    pub fn from_view<K: Layout>(
        device: &wgpu::Device,
        layout_storage: &mut LayoutStorage,
        texture_view: &wgpu::TextureView,
        layout: K,
    ) -> Self {
        let entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(texture_view),
        }];

        let layout = layout_storage.create_layout(device, layout);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some("Texture Bind Group"),
        });

        Self { bind_group }
    }
}
