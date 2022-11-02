use crate::graphics::{Atlas, Layout, LayoutStorage, TextureLayout};

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub enum GroupType {
    #[default]
    Textures,
    Fonts,
}

pub struct TextureGroup {
    pub bind_group: wgpu::BindGroup,
}

impl TextureGroup {
    pub fn from_view<K: Layout>(
        device: &wgpu::Device,
        layout_storage: &mut LayoutStorage,
        texture_view: &wgpu::TextureView,
        layout: K,
        texture_type: GroupType,
    ) -> Self {
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: if texture_type == GroupType::Textures {
                Some("Texture_sampler")
            } else {
                Some("Font_Texture_sampler")
            },
            lod_max_clamp: if texture_type == GroupType::Textures {
                1.0
            } else {
                0f32
            },
            ..Default::default()
        });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ];

        let layout = layout_storage.create_layout(device, layout);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: if texture_type == GroupType::Textures {
                Some("Texture Bind Group")
            } else {
                Some("Font Texture Bind Group")
            },
            layout: &layout,
            entries: &entries,
        });

        Self { bind_group }
    }
}
