use crate::{
    Allocation, Atlas, GroupType, LayoutStorage, TextureGroup, TextureLayout,
};
use std::{hash::Hash, time::Duration};

/// Group of a Atlas Details
pub struct AtlasGroup<U: Hash + Eq + Clone = String, Data: Copy + Default = i32>
{
    /// Atlas to hold Image locations
    pub atlas: Atlas<U, Data>,
    /// Texture Bind group for Atlas
    pub texture: TextureGroup,
}

impl<U: Hash + Eq + Clone, Data: Copy + Default> AtlasGroup<U, Data> {
    pub fn new(
        device: &wgpu::Device,
        size: u32,
        format: wgpu::TextureFormat,
        layout_storage: &mut LayoutStorage,
        group_type: GroupType,
        cache_start: usize,
        cache_duration: Duration,
    ) -> Self {
        let atlas = Atlas::<U, Data>::new(
            device,
            size,
            format,
            cache_start,
            cache_duration,
        );

        let texture = TextureGroup::from_view(
            device,
            layout_storage,
            &atlas.texture_view,
            TextureLayout,
            group_type,
        );

        Self { atlas, texture }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        hash: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Allocation<Data>> {
        self.atlas
            .upload(hash, bytes, width, height, data, device, queue)
    }
}
