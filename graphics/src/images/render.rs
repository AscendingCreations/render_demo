use crate::{
    AtlasGroup, ImageRenderPipeline, ImageVertex, InstanceBuffer,
    StaticBufferObject,
};

pub trait RenderImage<'a, 'b>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        buffer: &'b InstanceBuffer<ImageVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b ImageRenderPipeline,
    );
}

impl<'a, 'b> RenderImage<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        buffer: &'b InstanceBuffer<ImageVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b ImageRenderPipeline,
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
