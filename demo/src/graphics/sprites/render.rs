use crate::graphics::{
    AtlasGroup, GpuBuffer, SpriteRenderPipeline, SpriteVertex,
};

pub trait RenderSprite<'a, 'b>
where
    'b: 'a,
{
    fn render_sprite(
        &mut self,
        buffer: &'b GpuBuffer<SpriteVertex>,
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
        buffer: &'b GpuBuffer<SpriteVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b SpriteRenderPipeline,
    ) {
        if buffer.vertex_count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_vertex_buffer(0, buffer.vertices(None));
            self.set_index_buffer(
                buffer.indices(None),
                wgpu::IndexFormat::Uint32,
            );
            self.set_pipeline(pipeline.render_pipeline());
            self.draw_indexed(0..buffer.index_count() as u32, 0, 0..1);
        }
    }
}
