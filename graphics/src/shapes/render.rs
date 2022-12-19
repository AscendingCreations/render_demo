use crate::{
    InstanceBuffer, ShapeRenderPipeline, ShapeVertex, StaticBufferObject,
};

pub trait RenderShape<'a, 'b>
where
    'b: 'a,
{
    fn render_shape(
        &mut self,
        buffer: &'b InstanceBuffer<ShapeVertex>,
        pipeline: &'b ShapeRenderPipeline,
    );
}

impl<'a, 'b> RenderShape<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_shape(
        &mut self,
        buffer: &'b InstanceBuffer<ShapeVertex>,
        pipeline: &'b ShapeRenderPipeline,
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
