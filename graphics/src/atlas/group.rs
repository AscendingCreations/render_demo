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

pub trait AtlasType<U: Hash + Eq + Clone, Data: Copy + Default> {
    #[allow(clippy::too_many_arguments)]
    fn upload(
        &mut self,
        hash: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        renderer: &GpuRenderer,
    ) -> Option<usize>;
    #[allow(clippy::too_many_arguments)]
    fn upload_with_alloc(
        &mut self,
        hash: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        renderer: &GpuRenderer,
    ) -> Option<(usize, Allocation<Data>)>;
    fn lookup(&self, hash: &U) -> Option<usize>;
    fn trim(&mut self);
    fn clear(&mut self);
    fn promote(&mut self, id: usize);
    fn promote_by_key(&mut self, key: U);
    fn peek(&mut self, id: usize) -> Option<&(Allocation<Data>, U)>;
    fn peek_by_key(&mut self, key: &U) -> Option<&(Allocation<Data>, U)>;
    fn contains(&mut self, id: usize) -> bool;
    fn contains_key(&mut self, key: &U) -> bool;
    fn get(&mut self, id: usize) -> Option<Allocation<Data>>;
    fn get_by_key(&mut self, key: &U) -> Option<Allocation<Data>>;
    fn remove_by_key(&mut self, key: &U) -> Option<usize>;
    fn remove(&mut self, id: usize) -> Option<usize>;
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
}

impl<U: Hash + Eq + Clone, Data: Copy + Default> AtlasType<U, Data>
    for AtlasGroup<U, Data>
{
    #[allow(clippy::too_many_arguments)]
    fn upload(
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
    fn upload_with_alloc(
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

    fn lookup(&self, hash: &U) -> Option<usize> {
        self.atlas.lookup(hash)
    }

    fn trim(&mut self) {
        self.atlas.trim();
    }

    fn clear(&mut self) {
        self.atlas.clear();
    }

    fn promote(&mut self, id: usize) {
        self.atlas.promote(id);
    }

    fn promote_by_key(&mut self, key: U) {
        self.atlas.promote_by_key(key);
    }

    fn peek(&mut self, id: usize) -> Option<&(Allocation<Data>, U)> {
        self.atlas.peek(id)
    }

    fn peek_by_key(&mut self, key: &U) -> Option<&(Allocation<Data>, U)> {
        self.atlas.peek_by_key(key)
    }

    fn contains(&mut self, id: usize) -> bool {
        self.atlas.contains(id)
    }

    fn contains_key(&mut self, key: &U) -> bool {
        self.atlas.contains_key(key)
    }

    fn get(&mut self, id: usize) -> Option<Allocation<Data>> {
        self.atlas.get(id)
    }

    fn get_by_key(&mut self, key: &U) -> Option<Allocation<Data>> {
        self.atlas.get_by_key(key)
    }

    fn remove_by_key(&mut self, key: &U) -> Option<usize> {
        self.atlas.remove_by_key(key)
    }

    // returns the layer id if removed otherwise None for everything else.
    fn remove(&mut self, id: usize) -> Option<usize> {
        self.atlas.remove(id)
    }
}
