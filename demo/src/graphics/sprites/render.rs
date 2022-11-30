use crate::graphics::{
    AtlasGroup, InstanceBuffer, SpriteRenderPipeline, SpriteVertex,
    StaticBufferObject,
};

pub trait RenderSprite<'a, 'b>
where
    'b: 'a,
{
    fn render_sprite(
        &mut self,
        buffer: &'b InstanceBuffer<SpriteVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b SpriteRenderPipeline,
    );
}

impl<'a, 'b> RenderSprite<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_sprite(
        &mut self,
        buffer: &'b InstanceBuffer<SpriteVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b SpriteRenderPipeline,
    ) {
        if buffer.count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
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
