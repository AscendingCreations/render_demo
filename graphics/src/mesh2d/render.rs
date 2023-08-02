use crate::{
    AsBufferPass, AscendingError, GpuBuffer, GpuRenderer, Mesh2D,
    Mesh2DRenderPipeline, Mesh2DVertex, OrderedIndex, SetBuffers, System,
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

pub trait RenderMesh2D<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
        system: &'b System<Controls>,
    );
}

impl<'a, 'b, Controls> RenderMesh2D<'a, 'b, Controls> for wgpu::RenderPass<'a>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_2dmeshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b Mesh2DRenderer,
        system: &'b System<Controls>,
    ) {
        //TODO Add new mesh handler to cycle correct buffers with index id's

        if !buffer.vbos.buffers.is_empty() {
            self.set_buffers(buffer.vbos.as_buffer_pass());
            self.set_pipeline(
                renderer.get_pipelines(Mesh2DRenderPipeline).unwrap(),
            );
            let mut scissor_is_default = true;
            let mut index_pos = 0;
            let mut base_vertex = 0;

            for (i, details) in buffer.vbos.buffers.iter().enumerate() {
                if let Some(Some(bounds)) = buffer.vbos.bounds.get(i) {
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
