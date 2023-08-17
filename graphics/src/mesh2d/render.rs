use crate::{
    AsBufferPass, AscendingError, GpuBuffer, GpuRenderer, Mesh2D,
    Mesh2DRenderPipeline, Mesh2DVertex, OrderedIndex, SetBuffers,
};

pub struct Mesh2DRenderer {
    pub vbos: GpuBuffer<Mesh2DVertex>,
}

//TODO: Update this to take in instance buffer index too.
impl Mesh2DRenderer {
    pub fn new(renderer: &GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            vbos: GpuBuffer::new(renderer.gpu_device()),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
    ) {
        self.vbos.add_buffer_store(renderer, index);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.vbos.finalize(renderer);
    }

    pub fn mesh_update(
        &mut self,
        mesh: &mut Mesh2D,
        renderer: &mut GpuRenderer,
    ) {
        let index = mesh.update(renderer);

        self.add_buffer_store(renderer, index);
    }
}

pub trait RenderMesh2D<'a, 'b>
where
    'b: 'a,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
    );
}

impl<'a, 'b> RenderMesh2D<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
    ) {
        if !buffer.vbos.buffers.is_empty() {
            self.set_buffers(buffer.vbos.as_buffer_pass());
            self.set_pipeline(
                renderer.get_pipelines(Mesh2DRenderPipeline).unwrap(),
            );
            let mut index_pos = 0;
            let mut base_vertex = 0;

            for details in &buffer.vbos.buffers {
                // Indexs can always start at 0 per mesh data.
                // Base vertex is the Addition to the Index
                self.draw_indexed(
                    index_pos..index_pos + details.count,
                    base_vertex, //i as i32 * details.max,
                    0..1,
                );

                base_vertex += details.max as i32 + 1;
                index_pos += details.count;
            }
        }
    }
}
