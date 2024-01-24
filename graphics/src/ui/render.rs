use crate::{
    AscendingError, Atlas, AtlasGroup, GpuRenderer, InstanceBuffer,
    OrderedIndex, Rect, RectRenderPipeline, RectVertex, StaticBufferObject,
};

pub struct RectRenderer {
    pub buffer: InstanceBuffer<RectVertex>,
}

impl RectRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device()),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
    ) {
        self.buffer.add_buffer_store(renderer, index);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer)
    }

    pub fn rect_update(
        &mut self,
        rect: &mut Rect,
        renderer: &mut GpuRenderer,
        atlas: &mut Atlas,
    ) {
        let index = rect.update(renderer, atlas);

        self.add_buffer_store(renderer, index);
    }
}

pub trait RenderRects<'a, 'b>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas_group: &'b AtlasGroup,
    );
}

impl<'a, 'b> RenderRects<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_rects(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b RectRenderer,
        atlas_group: &'b AtlasGroup,
    ) {
        if buffer.buffer.count() > 0 {
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.buffer.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(RectRenderPipeline).unwrap(),
            );

            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..buffer.buffer.count(),
            );
        }
    }
}
