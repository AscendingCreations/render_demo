use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;

pub struct GuiRender {
    /// Basic shape rendering.
    pub ui: Rect,
    pub ui_buffer: InstanceBuffer<RectVertex>,
    pub ui_pipeline: RectsRenderPipeline,
    pub ui_atlas: AtlasGroup,
    /// Text test stuff.
    pub text_render: TextRender,
    pub text_buffer: InstanceBuffer<TextVertex>,
    pub text_pipeline: TextRenderPipeline,
    pub text_atlas: AtlasGroup<CacheKey, (i32, i32)>,
    pub emoji_atlas: AtlasGroup<CacheKey, (i32, i32)>,
}
