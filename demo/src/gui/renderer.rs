use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;

pub struct GuiRender {
    /// Sprite data TODO: Make an array,
    pub sprites: Vec<Image>,
    /// Render pipe line for Sprites
    pub sprite_pipeline: ImageRenderPipeline,
    /// Vertex buffer group for Sprites
    pub sprite_buffer: InstanceBuffer<ImageVertex>,
    /// AtlasGroup to hold Sprite Images
    pub sprite_atlas: AtlasGroup,
    /// maps TODO: make this an array.
    pub map: Map,
    /// Render Pipeline for maps
    pub map_pipeline: MapRenderPipeline,
    /// vertex buffer group for maps
    pub maplower_buffer: InstanceBuffer<MapVertex>,
    pub mapupper_buffer: InstanceBuffer<MapVertex>,
    /// Texture Bind group for Maptextures
    pub map_group: TextureGroup,
    /// contains the Map layer grids in pixel form.
    pub map_textures: MapTextures,
    /// contains the Tile images.
    pub map_atlas: AtlasGroup,
    /// animation test stuff.
    pub animation: Image,
    pub animation_buffer: InstanceBuffer<ImageVertex>,
    pub animation_atlas: AtlasGroup,

    /// Basic shape rendering.
    pub rects: Rectangles,
    pub rects_buffer: InstanceBuffer<RectVertex>,
    pub rects_pipeline: RectsRenderPipeline,

    /// Text test stuff.
    pub text: Text,
    pub text_buffer: InstanceBuffer<TextVertex>,
    pub text_pipeline: TextRenderPipeline,
    pub text_atlas: AtlasGroup<CacheKey, (i32, i32)>,
    pub emoji_atlas: AtlasGroup<CacheKey, (i32, i32)>,
    pub buffer_object: StaticBufferObject,
}
