pub(crate) use crate::graphics::{
    AtlasGroup, GpuBuffer, MapRenderPipeline, MapVertex, TextureGroup,
};

pub trait RenderMap<'a, 'b>
where
    'b: 'a,
{
    fn render_maps(
        &mut self,
        buffer: &'b GpuBuffer<MapVertex>,
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
        buffer: &'b GpuBuffer<MapVertex>,
        atlas_group: &'b AtlasGroup,
        map_group: &'b TextureGroup,
        pipeline: &'b MapRenderPipeline,
    ) {
        if buffer.vertex_count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_bind_group(2, &map_group.bind_group, &[]);
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
