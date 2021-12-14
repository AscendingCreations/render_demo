use crate::graphics::{
    AnimationRenderPipeline, AnimationVertex, GpuBuffer, TextureGroup,
};

pub trait RenderAnimations<'a, 'b>
where
    'b: 'a,
{
    fn render_animations(
        &mut self,
        buffer: &'b GpuBuffer<AnimationVertex>,
        texture_group: &'b TextureGroup,
        pipeline: &'b AnimationRenderPipeline,
    );
}

impl<'a, 'b> RenderAnimations<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_animations(
        &mut self,
        buffer: &'b GpuBuffer<AnimationVertex>,
        texture_group: &'b TextureGroup,
        pipeline: &'b AnimationRenderPipeline,
    ) {
        self.set_bind_group(2, &texture_group.bind_group, &[]);
        self.set_vertex_buffer(0, buffer.vertices(None));
        self.set_index_buffer(buffer.indices(None), wgpu::IndexFormat::Uint32);
        self.set_pipeline(pipeline.render_pipeline());
        self.draw_indexed(0..buffer.index_count() as u32, 0, 0..1);
    }
}
