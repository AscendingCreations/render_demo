use crate::{
    AscendingError, AtlasGroup, GpuRenderer, GroupType, InstanceBuffer,
    OrderedIndex, StaticBufferObject, Text, TextRenderPipeline, TextVertex,
    Vec2,
};
use cosmic_text::{CacheKey, FontSystem, SwashCache};

pub struct TextAtlas {
    pub(crate) text: AtlasGroup<CacheKey, Vec2>,
    pub(crate) emoji: AtlasGroup<CacheKey, Vec2>,
}

impl TextAtlas {
    pub fn new(
        renderer: &mut GpuRenderer,
        min_pressure: usize,
        max_pressure: usize,
        size: u32,
    ) -> Result<Self, AscendingError> {
        Ok(Self {
            text: AtlasGroup::new(
                renderer,
                size,
                wgpu::TextureFormat::R8Unorm,
                GroupType::Fonts,
                min_pressure,
                max_pressure,
            ),
            emoji: AtlasGroup::new(
                renderer,
                size,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                GroupType::Textures,
                min_pressure,
                max_pressure,
            ),
        })
    }

    pub fn clean(&mut self) {
        self.emoji.clean();
        self.text.clean();
    }
}

pub struct TextRenderer {
    pub(crate) buffer: InstanceBuffer<TextVertex>,
    pub(crate) pipeline: TextRenderPipeline,
    pub(crate) swash_cache: SwashCache,
}

impl TextRenderer {
    pub fn new(
        renderer: &mut GpuRenderer,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device()),
            pipeline: TextRenderPipeline::new(renderer, surface_format)?,
            swash_cache: SwashCache::new(),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &mut GpuRenderer,
        index: OrderedIndex,
    ) {
        self.buffer.add_buffer_store(renderer, index);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer)
    }

    pub fn text_update(
        &mut self,
        text: &mut Text,
        atlas: &mut TextAtlas,
        font_system: &FontSystem,
        renderer: &mut GpuRenderer,
    ) -> Result<(), AscendingError> {
        let index =
            text.update(font_system, &mut self.swash_cache, atlas, renderer)?;

        self.add_buffer_store(renderer, index);
        Ok(())
    }
}

pub trait RenderText<'a, 'b>
where
    'b: 'a,
{
    fn render_text(&mut self, renderer: &'b TextRenderer, atlas: &'b TextAtlas);
}

impl<'a, 'b> RenderText<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        renderer: &'b TextRenderer,
        atlas: &'b TextAtlas,
    ) {
        if renderer.buffer.count() > 0 {
            self.set_bind_group(1, &atlas.text.texture.bind_group, &[]);
            self.set_bind_group(2, &atlas.emoji.texture.bind_group, &[]);
            self.set_vertex_buffer(1, renderer.buffer.instances(None));
            self.set_pipeline(renderer.pipeline.render_pipeline());
            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..renderer.buffer.count(),
            );
        }
    }
}
