pub(crate) use crate::graphics::{GpuBuffer, ShapeRenderPipeline, ShapeVertex};

pub trait RenderShape<'a, 'b>
where
    'b: 'a,
{
    fn render_shape(
        &mut self,
        buffer: &'b GpuBuffer<ShapeVertex>,
        pipeline: &'b ShapeRenderPipeline,
    );
}

impl<'a, 'b> RenderShape<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_shape(
        &mut self,
        buffer: &'b GpuBuffer<ShapeVertex>,
        pipeline: &'b ShapeRenderPipeline,
    ) {
        self.set_vertex_buffer(0, buffer.vertices(None));
        self.set_index_buffer(buffer.indices(None), wgpu::IndexFormat::Uint32);
        self.set_pipeline(pipeline.render_pipeline());
        self.draw_indexed(0..buffer.index_count() as u32, 0, 0..1);
    }
}
