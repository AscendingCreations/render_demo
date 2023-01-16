use crate::{
    AtlasGroup, InstanceBuffer, RectVertex, RectsRenderPipeline,
    StaticBufferObject,
};

pub trait RenderRects<'a, 'b>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        buffer: &'b InstanceBuffer<RectVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b RectsRenderPipeline,
    );
}

impl<'a, 'b> RenderRects<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        buffer: &'b InstanceBuffer<RectVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b RectsRenderPipeline,
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
