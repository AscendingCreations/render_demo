use crate::{
    AtlasGroup, GpuDevice, GpuWindow, InstanceBuffer, LayoutStorage,
    StaticBufferObject, TextRenderPipeline, TextVertex, Vec2,
};
use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;

pub struct UIBuffer {
    /// Basic shape/image rendering for widgets.
    pub ui_buffer: InstanceBuffer<RectVertex>,
    pub ui_pipeline: RectsRenderPipeline,
    pub ui_atlas: AtlasGroup,
    /// Text test stuff.
    pub text_buffer: InstanceBuffer<TextVertex>,
    pub text_pipeline: TextRenderPipeline,
    pub text_atlas: AtlasGroup<CacheKey, Vec2>,
    pub emoji_atlas: AtlasGroup<CacheKey, Vec2>,
}

impl UIBuffer {
    pub fn new(
        gpu_device: &GpuDevice,
        gpu_window: &GpuWindow,
        layout_storage: &mut LayoutStorage,
    ) -> Result<Self, AscendingError> {
        Ok(Self {
            ui_buffer: InstanceBuffer::new(gpu_device),
            ui_pipeline: RectsRenderPipeline::new(
                gpu_device,
                gpu_window.surface_format(),
                layout_storage,
            )?,
            ui_atlas: AtlasGroup::new(
                gpu_device,
                2048,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                layout_storage,
                GroupType::Textures,
                256,
                256,
            ),
            text_buffer: InstanceBuffer::new(gpu_device),
            text_pipeline: TextRenderPipeline::new(
                gpu_device,
                gpu_window.surface_format(),
                layout_storage,
            )?,
            text_atlas: AtlasGroup::new(
                gpu_device,
                2048,
                wgpu::TextureFormat::R8Unorm,
                layout_storage,
                GroupType::Fonts,
                2,
                256,
            ),
            emoji_atlas: AtlasGroup::new(
                gpu_device,
                2048,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                layout_storage,
                GroupType::Textures,
                2,
                256,
            ),
        })
    }

    pub fn atlas_clean(&mut self) {
        self.emoji_atlas.clean();
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
        buffer: &'b UIBuffer,
        system: &'b System<Controls>,
    ) {
        self.render_rects(
            &buffer.ui_buffer,
            &buffer.ui_atlas,
            &buffer.ui_pipeline,
            system,
        );

        self.render_text(
            &buffer.text_buffer,
            &buffer.text_atlas,
            &buffer.emoji_atlas,
            &buffer.text_pipeline,
        );
    }
}
