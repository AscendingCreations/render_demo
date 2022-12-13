use crate::graphics::{
    Allocation, Atlas, GroupType, LayoutStorage, Texture, TextureGroup,
    TextureLayout,
};
use std::{collections::HashMap, hash::Hash};

/// Group of a Atlas Details
pub struct AtlasGroup<U: Hash + Eq + Clone = String, Data: Copy + Default = i32>
{
    /// Atlas to hold Image locations
    pub atlas: Atlas<U>,
    /// Texture Bind group for Atlas
    pub texture: TextureGroup,
    //Store any Extra data per Allocation.
    pub data: HashMap<U, Data>,
}

impl<U: Hash + Eq + Clone, Data: Copy + Default> AtlasGroup<U, Data> {
    pub fn new(
        device: &wgpu::Device,
        size: u32,
        format: wgpu::TextureFormat,
        layout_storage: &mut LayoutStorage,
        group_type: GroupType,
    ) -> Self {
        let atlas = Atlas::<U>::new(device, size, format);

        let texture = TextureGroup::from_view(
            device,
            layout_storage,
            &atlas.texture_view,
            TextureLayout,
            group_type,
        );

        Self {
            atlas,
            texture,
            data: HashMap::new(),
        }
    }

    pub fn upload(
        &mut self,
        hash: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Allocation> {
        self.atlas.upload(hash, bytes, width, height, device, queue)
    }

    pub fn upload_data(&mut self, hash: U, data: Data) {
        self.data.insert(hash, data);
    }

    pub fn get_data(&mut self, hash: &U) -> Option<Data> {
        self.data.get(hash).copied()
    }
}
