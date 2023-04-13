use crate::{
    AscendingError, AtlasGroup, GpuRenderer, InstanceBuffer, OrderedIndex,
    Rect, RectVertex, RectsRenderPipeline, StaticBufferObject, System,
};

pub struct RectRenderer {
    pub buffer: InstanceBuffer<RectVertex>,
}

impl RectRenderer {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device()),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &mut GpuRenderer,
        index: OrderedIndex,
    ) {
        self.buffer.add_buffer_store(renderer, index);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer)
    }

    pub fn rect_update(&mut self, rect: &mut Rect, renderer: &mut GpuRenderer) {
        let index = rect.update(renderer);

        self.add_buffer_store(renderer, index);
    }
}

pub trait RenderRects<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas_group: &'b AtlasGroup,
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
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas_group: &'b AtlasGroup,
        system: &'b System<Controls>,
    ) {
        if buffer.buffer.count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.buffer.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(RectsRenderPipeline).unwrap(),
            );
            let mut scissor_is_default = true;

            for i in 0..buffer.buffer.count() {
                if let Some(Some(bounds)) = buffer.buffer.bounds.get(i as usize)
                {
                    let bounds = system.world_to_screen(false, bounds);

                    self.set_scissor_rect(
                        bounds.x as u32,
                        bounds.y as u32,
                        bounds.z as u32,
                        bounds.w as u32,
                    );
                    scissor_is_default = false;
                } else if !scissor_is_default {
                    self.set_scissor_rect(
                        0,
                        0,
                        system.screen_size[0] as u32,
                        system.screen_size[1] as u32,
                    );
                    scissor_is_default = true;
                };

                self.draw_indexed(
                    0..StaticBufferObject::index_count(),
                    0,
                    i..i + 1,
                );
            }

            //Gotta set it back otherwise it will clip everything after it...
            if !scissor_is_default {
                self.set_scissor_rect(
                    0,
                    0,
                    system.screen_size[0] as u32,
                    system.screen_size[1] as u32,
                );
            }
        }
    }
}
