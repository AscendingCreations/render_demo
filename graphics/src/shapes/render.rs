use crate::{
    InstanceBuffer, RectVertex, RectsRenderPipeline, StaticBufferObject,
};

pub trait RenderRects<'a, 'b>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        buffer: &'b InstanceBuffer<RectVertex>,
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
        pipeline: &'b RectsRenderPipeline,
    ) {
        if buffer.count() > 0 {
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
