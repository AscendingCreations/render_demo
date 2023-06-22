use crate::{
    AsBufferPass, AscendingError, AtlasGroup, GpuBuffer, GpuRenderer,
    InstanceBuffer, Mesh, MeshInstance, MeshRenderPipeline, MeshVertex,
    OrderedIndex, System,
};

pub struct MeshRenderer {
    pub instances: InstanceBuffer<MeshInstance>,
    pub vbos: GpuBuffer<MeshVertex>,
}

pub struct MeshOrderIndex {
    pub vbo: OrderedIndex,
    pub ibo: OrderedIndex,
}

//TODO: Update this to take in instance buffer index too.
impl MeshRenderer {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        Ok(Self {
            instances: InstanceBuffer::new(renderer.gpu_device()),
            vbos: GpuBuffer::new(renderer.gpu_device()),
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &mut GpuRenderer,
        index: MeshOrderIndex,
    ) {
        self.instances.add_buffer_store(renderer, index.ibo);
        self.vbos.add_buffer_store(renderer, index.vbo);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.instances.finalize(renderer);
        self.vbos.finalize(renderer);
    }

    pub fn mesh_update(&mut self, mesh: &mut Mesh, renderer: &mut GpuRenderer) {
        let index = mesh.update(renderer);

        self.add_buffer_store(renderer, index);
    }
}

pub trait RenderMesh<'a, 'b, Controls>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_meshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b MeshRenderer,
        atlas_group: &'b AtlasGroup,
        system: &'b System<Controls>,
    );
}

impl<'a, 'b, Controls> RenderMesh<'a, 'b, Controls> for wgpu::RenderPass<'a>
where
    'b: 'a,
    Controls: camera::controls::Controls,
{
    fn render_meshs(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b MeshRenderer,
        atlas_group: &'b AtlasGroup,
        system: &'b System<Controls>,
    ) {
        //TODO Add new mesh handler to cycle correct buffers with index id's
        /*
        if buffer.vbos.count() > 0 {
            self.set_buffers(buffer.vbos.as_buffer_send());
            self.set_bind_group(1, &atlas_group.texture.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.instances.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(MeshRenderPipeline).unwrap(),
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

                // Indexs can always start at 0 per mesh data.
                // Base vertex is the Addition to the Index
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
        }*/
    }
}
