use crate::{
    AscendingError, AtlasGroup, GpuRenderer, Image, ImageRenderPipeline,
    ImageVertex, InstanceBuffer, OrderedIndex, StaticBufferObject,
};

pub struct ImageRenderer {
    pub pipeline: ImageRenderPipeline,
    pub buffer: InstanceBuffer<ImageVertex>,
}

impl ImageRenderer {
    pub fn new(
        renderer: &mut GpuRenderer,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, AscendingError> {
        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device()),
            pipeline: ImageRenderPipeline::new(renderer, surface_format)?,
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

    pub fn image_update(
        &mut self,
        image: &mut Image,
        renderer: &mut GpuRenderer,
    ) {
        let index = image.update(renderer);

        self.add_buffer_store(renderer, index);
    }
}

pub trait RenderImage<'a, 'b>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        renderer: &'b ImageRenderer,
        atlas: &'b AtlasGroup,
    );
}

impl<'a, 'b> RenderImage<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        renderer: &'b ImageRenderer,
        atlas: &'b AtlasGroup,
    ) {
        if renderer.buffer.count() > 0 {
            self.set_bind_group(1, &atlas.texture.bind_group, &[]);
            self.set_vertex_buffer(1, renderer.buffer.instances(None));
            self.set_pipeline(renderer.pipeline.render_pipeline());

            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..renderer.buffer.count(),
            );
        }
    }
}
