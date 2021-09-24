use crate::graphics::{SpriteBuffer, SpriteRenderPipeline, TextureGroup};

pub trait RenderSprite<'a, 'b>
where
    'b: 'a,
{
    fn render_sprite(
        &mut self,
        buffer: &'b SpriteBuffer,
        texture_group: &'b TextureGroup,
        pipeline: &'b SpriteRenderPipeline,
    );
}

impl<'a, 'b> RenderSprite<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_sprite(
        &mut self,
        buffer: &'b SpriteBuffer,
        texture_group: &'b TextureGroup,
        pipeline: &'b SpriteRenderPipeline,
    ) {
        self.set_pipeline(pipeline.render_pipeline());
        self.set_bind_group(1, &texture_group.bind_group, &[]);
        self.set_vertex_buffer(0, buffer.vertex_buffer.slice(..buffer.vertex_size));
        self.set_index_buffer(
            buffer.indice_buffer.slice(..buffer.num_indices as u64),
            wgpu::IndexFormat::Uint32,
        );
        self.draw_indexed(0..(buffer.num_indices as u32), 0, 0..1);
    }
}
