use crate::{
    AscendingError, AtlasGroup, GpuRenderer, Image, ImageRenderPipeline,
    ImageVertex, InstanceBuffer, OrderedIndex, StaticBufferObject,
};

pub struct ImageRenderer {
    pub buffer: InstanceBuffer<ImageVertex>,
}

impl ImageRenderer {
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
        renderer: &'b GpuRenderer,
        buffer: &'b ImageRenderer,
        atlas: &'b AtlasGroup,
    );
}

impl<'a, 'b> RenderImage<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_image(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b ImageRenderer,
        atlas: &'b AtlasGroup,
    ) {
        if buffer.buffer.count() > 0 {
            self.set_bind_group(1, &atlas.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.buffer.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(ImageRenderPipeline).unwrap(),
            );

            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..buffer.buffer.count(),
            );
        }
    }
}
