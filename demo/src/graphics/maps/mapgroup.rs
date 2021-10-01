use crate::graphics::{LayoutStorage, Map, MapLayout};

pub struct MapGroup {
    pub bind_group: wgpu::BindGroup,
}

impl MapGroup {
    pub fn from_maps(
        device: &wgpu::Device,
        layout_storage: &mut LayoutStorage,
        maps: &mut [Map],
    ) -> Self {
        /*let mut map_arr = Vec::new();

        for (i, map) in maps.iter_mut().enumerate() {
            map_arr.push(&map.texture_view);
            map.array_id = i as u32;
        }*/

        let entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&maps[0].texture_view),
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
