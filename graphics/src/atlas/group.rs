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
        use_ref_count: bool,
    ) -> Self {
        let atlas = Atlas::<U, Data>::new(renderer, format, use_ref_count);

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
    ) -> Option<usize> {
        self.atlas
            .upload(hash, bytes, width, height, data, renderer)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload_with_alloc(
        &mut self,
        hash: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        renderer: &GpuRenderer,
    ) -> Option<(usize, Allocation<Data>)> {
        self.atlas
            .upload_with_alloc(hash, bytes, width, height, data, renderer)
    }

    pub fn trim(&mut self) {
        self.atlas.trim();
    }

    pub fn clear(&mut self) {
        self.atlas.clear();
    }

    pub fn promote(&mut self, id: usize) {
        self.atlas.promote(id);
    }

    pub fn promote_by_key(&mut self, key: U) {
        self.atlas.promote_by_key(key);
    }

    pub fn peek(&mut self, id: usize) -> Option<&(Allocation<Data>, U)> {
        self.atlas.peek(id)
    }

    pub fn peek_by_key(&mut self, key: &U) -> Option<&(Allocation<Data>, U)> {
        self.atlas.peek_by_key(key)
    }

    pub fn contains(&mut self, id: usize) -> bool {
        self.atlas.contains(id)
    }

    pub fn contains_key(&mut self, key: &U) -> bool {
        self.atlas.contains_key(key)
    }

    pub fn get(&mut self, id: usize) -> Option<Allocation<Data>> {
        self.atlas.get(id)
    }

    pub fn get_by_key(&mut self, key: &U) -> Option<Allocation<Data>> {
        self.atlas.get_by_key(key)
    }
}
