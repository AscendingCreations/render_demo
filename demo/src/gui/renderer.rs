use crate::{
    AtlasGroup, GpuRenderer, InstanceBuffer, LayoutStorage, StaticBufferObject,
    TextAtlas, TextRenderPipeline, TextVertex, UiRenderer, Vec2,
};
use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;

pub struct UIBuffer {
    /// Basic shape/image rendering for widgets.
    pub ui_buffer: UiRenderer,
    pub ui_atlas: AtlasGroup,
    /// Text test stuff.
    pub text_renderer: TextRenderer,
    pub text_atlas: TextAtlas,
}

impl UIBuffer {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            ui_buffer: UiRenderer::new(renderer)?,
            ui_atlas: AtlasGroup::new(
                renderer,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ),
            text_renderer: TextRenderer::new(renderer)?,
            text_atlas: TextAtlas::new(renderer)?,
        })
    }

    pub fn atlas_trim(&mut self) {
        self.text_atlas.trim();
        self.ui_atlas.trim();
    }
}

pub trait RenderWidgets<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_widgets(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b UIBuffer,
        system: &'b System<Controls>,
    );
}

impl<'a, 'b, Controls> RenderWidgets<'a, 'b, Controls> for wgpu::RenderPass<'a>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_widgets(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b UIBuffer,
        system: &'b System<Controls>,
    ) {
        self.render_rects(
            renderer,
            &buffer.ui_buffer,
            &buffer.ui_atlas,
            system,
        );

        self.render_text(renderer, &buffer.text_renderer, &buffer.text_atlas);
    }
}
