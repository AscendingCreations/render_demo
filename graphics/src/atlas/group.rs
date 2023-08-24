use crate::{Allocation, Atlas, GpuRenderer, TextureGroup, TextureLayout};
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
        renderer: &mut GpuRenderer,
        format: wgpu::TextureFormat,
    ) -> Self {
        let atlas = Atlas::<U, Data>::new(renderer, format);

        let texture = TextureGroup::from_view(
            renderer,
            &atlas.texture_view,
            TextureLayout,
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
        renderer: &GpuRenderer,
    ) -> Option<Allocation<Data>> {
        self.atlas
            .upload(hash, bytes, width, height, data, renderer)
    }

    pub fn trim(&mut self) {
        self.atlas.trim();
    }

    pub fn clear(&mut self) {
        self.atlas.clear();
    }

    pub fn promote(&mut self, key: U) {
        self.atlas.promote(key);
    }

    pub fn peek(&mut self, key: &U) -> Option<&Allocation<Data>> {
        self.atlas.peek(key)
    }

    pub fn contains(&mut self, key: &U) -> bool {
        self.atlas.contains(key)
    }

    pub fn get(&mut self, key: &U) -> Option<Allocation<Data>> {
        self.atlas.get(key)
    }
}
