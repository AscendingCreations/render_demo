use crate::{
    AtlasGroup, GpuRenderer, InstanceBuffer, LayoutStorage, RectRenderer,
    StaticBufferObject, TextAtlas, TextRenderPipeline, TextVertex, Vec2,
};
use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;

pub struct UIBuffer {
    /// Basic shape/image rendering for widgets.
    pub ui_buffer: RectRenderer,
    pub ui_atlas: AtlasGroup,
    /// Text test stuff.
    pub text_renderer: TextRenderer,
    pub text_atlas: TextAtlas,
}

impl UIBuffer {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            ui_buffer: RectRenderer::new(renderer)?,
            ui_atlas: AtlasGroup::new(
                renderer,
                2048,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                GroupType::Textures,
                256,
                256,
            ),
            text_renderer: TextRenderer::new(renderer)?,
            text_atlas: TextAtlas::new(renderer, 2, 256, 2048)?,
        })
    }

    pub fn atlas_clean(&mut self) {
        self.text_atlas.clean();
        self.ui_atlas.clean();
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
