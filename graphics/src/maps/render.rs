use crate::{
    AtlasGroup, InstanceBuffer, MapRenderPipeline, MapVertex,
    StaticBufferObject, TextureGroup,
};

pub trait RenderMap<'a, 'b>
where
    'b: 'a,
{
    fn render_maps(
        &mut self,
        buffer: &'b InstanceBuffer<MapVertex>,
        atlas_group: &'b AtlasGroup,
        map_group: &'b TextureGroup,
        pipeline: &'b MapRenderPipeline,
    );
}

impl<'a, 'b> RenderMap<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_maps(
        &mut self,
        buffer: &'b InstanceBuffer<MapVertex>,
        atlas_group: &'b AtlasGroup,
        map_group: &'b TextureGroup,
        pipeline: &'b MapRenderPipeline,
    ) {
        if buffer.count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_bind_group(2, &map_group.bind_group, &[]);
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
