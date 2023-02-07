use crate::{
    AtlasGroup, InstanceBuffer, RectVertex, RectsRenderPipeline,
    StaticBufferObject, System,
};

pub trait RenderRects<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_rects(
        &mut self,
        buffer: &'b InstanceBuffer<RectVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b RectsRenderPipeline,
        system: &'b System<Controls>,
    );
}

impl<'a, 'b, Controls> RenderRects<'a, 'b, Controls> for wgpu::RenderPass<'a>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_rects(
        &mut self,
        buffer: &'b InstanceBuffer<RectVertex>,
        atlas_group: &'b AtlasGroup,
        pipeline: &'b RectsRenderPipeline,
        system: &'b System<Controls>,
    ) {
        if buffer.count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.instances(None));
            self.set_pipeline(pipeline.render_pipeline());

            for i in 0..buffer.count() {
                if let Some(Some(bounds)) = buffer.bounds.get(i as usize) {
                    let bounds = system.world_to_screen(false, &bounds);

                    self.set_scissor_rect(
                        bounds.x as u32,
                        bounds.y as u32,
                        bounds.w as u32,
                        bounds.h as u32,
                    );
                } else {
                    self.set_scissor_rect(
                        0,
                        0,
                        system.screen_size[0] as u32,
                        system.screen_size[1] as u32,
                    );
                };

                self.draw_indexed(
                    0..StaticBufferObject::index_count(),
                    0,
                    i..i + 1,
                );
            }

            //Gotta set it back otherwise it will clip everything after it...
            self.set_scissor_rect(
                0,
                0,
                system.screen_size[0] as u32,
                system.screen_size[1] as u32,
            );
        }
    }
}
