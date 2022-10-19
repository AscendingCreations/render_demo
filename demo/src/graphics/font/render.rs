pub(crate) use crate::graphics::{
    GpuBuffer, ScreenGroup, TextRenderPipeline, TextVertex, TextureGroup,
};

pub trait RenderText<'a, 'b>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        buffer: &'b GpuBuffer<TextVertex>,
        texture_group: &'b TextureGroup,
        pipeline: &'b TextRenderPipeline,
        screen_group: &'b ScreenGroup,
    );
}

impl<'a, 'b> RenderText<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_text(
        &mut self,
        buffer: &'b GpuBuffer<TextVertex>,
        texture_group: &'b TextureGroup,
        pipeline: &'b TextRenderPipeline,
        screen_group: &'b ScreenGroup,
    ) {
        if buffer.vertex_count() > 0 {
            self.set_bind_group(2, screen_group.bind_group(), &[]);
            self.set_bind_group(3, &texture_group.bind_group, &[]);
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
