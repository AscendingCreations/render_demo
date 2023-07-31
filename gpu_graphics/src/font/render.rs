use crate::{
    AsBufferPass, AscendingError, AtlasGroup, GpuRenderer, InstanceBuffer,
    OrderedIndex, SetBuffers, StaticBufferObject, Text, TextRenderPipeline,
    TextVertex, Vec2,
};
use cosmic_text::{CacheKey, SwashCache};

pub struct TextAtlas {
    pub(crate) text: AtlasGroup<CacheKey, Vec2>,
    pub(crate) emoji: AtlasGroup<CacheKey, Vec2>,
}

impl TextAtlas {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            text: AtlasGroup::new(renderer, wgpu::TextureFormat::R8Unorm),
            emoji: AtlasGroup::new(
                renderer,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ),
        })
    }

    pub fn trim(&mut self) {
        self.emoji.trim();
        self.text.trim();
    }
}

pub struct TextRenderer {
    pub(crate) buffer: InstanceBuffer<TextVertex>,
    pub(crate) swash_cache: SwashCache,
}

impl TextRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device()),
            swash_cache: SwashCache::new(),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
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
        renderer: &mut GpuRenderer,
    ) -> Result<(), AscendingError> {
        let index = text.update(&mut self.swash_cache, atlas, renderer)?;

        self.add_buffer_store(renderer, index);
        Ok(())
    }
}

pub trait RenderText<'a, 'b>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b TextRenderer,
        atlas: &'b TextAtlas,
    );
}

impl<'a, 'b> RenderText<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b TextRenderer,
        atlas: &'b TextAtlas,
    ) {
        if buffer.buffer.count() > 0 {
            self.set_buffers(renderer.buffer_object.as_buffer_pass());
            self.set_bind_group(1, &atlas.text.texture.bind_group, &[]);
            self.set_bind_group(2, &atlas.emoji.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.buffer.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(TextRenderPipeline).unwrap(),
            );
            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..buffer.buffer.count(),
            );
        }
    }
}
