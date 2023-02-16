use crate::{
    AtlasGroup, InstanceBuffer, StaticBufferObject, TextRenderPipeline,
    TextVertex, Vec2,
};
use cosmic_text::CacheKey;

pub trait RenderText<'a, 'b>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        buffer: &'b InstanceBuffer<TextVertex>,
        text_atlas_group: &'b AtlasGroup<CacheKey, Vec2>,
        emoji_atlas_group: &'b AtlasGroup<CacheKey, Vec2>,
        pipeline: &'b TextRenderPipeline,
    );
}

impl<'a, 'b> RenderText<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        buffer: &'b InstanceBuffer<TextVertex>,
        text_atlas_group: &'b AtlasGroup<CacheKey, Vec2>,
        emoji_atlas_group: &'b AtlasGroup<CacheKey, Vec2>,
        pipeline: &'b TextRenderPipeline,
    ) {
        if buffer.count() > 0 {
            self.set_bind_group(1, &text_atlas_group.texture.bind_group, &[]);
            self.set_bind_group(2, &emoji_atlas_group.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.instances(None));
            self.set_pipeline(pipeline.render_pipeline());
            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..buffer.count(),
            );
        }
    }
}
