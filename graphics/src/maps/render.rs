use crate::{
    AtlasGroup, GpuRenderer, InstanceBuffer, MapRenderPipeline, MapVertex,
    StaticBufferObject, TextureGroup,
};

pub trait RenderMap<'a, 'b>
where
    'b: 'a,
{
    fn render_maps(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b InstanceBuffer<MapVertex>,
        atlas_group: &'b AtlasGroup,
        map_group: &'b TextureGroup,
    );
}

impl<'a, 'b> RenderMap<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_maps(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b InstanceBuffer<MapVertex>,
        atlas_group: &'b AtlasGroup,
        map_group: &'b TextureGroup,
    ) {
        if buffer.count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_bind_group(2, &map_group.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(MapRenderPipeline).unwrap(),
            );
            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..buffer.count(),
            );
        }
    }
}
