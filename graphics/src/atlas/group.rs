use crate::{
    Allocation, Atlas, GpuDevice, GroupType, LayoutStorage, TextureGroup,
    TextureLayout,
};
use std::hash::Hash;

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
        gpu_device: &GpuDevice,
        size: u32,
        format: wgpu::TextureFormat,
        layout_storage: &mut LayoutStorage,
        group_type: GroupType,
        pressure_min: usize,
        pressure_max: usize,
    ) -> Self {
        let atlas = Atlas::<U, Data>::new(
            gpu_device,
            size,
            format,
            pressure_min,
            pressure_max,
        );

        let texture = TextureGroup::from_view(
            gpu_device,
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
        gpu_device: &GpuDevice,
    ) -> Option<Allocation<Data>> {
        self.atlas
            .upload(hash, bytes, width, height, data, gpu_device)
    }

    pub fn clean(&mut self) {
        self.atlas.clean();
    }
}
