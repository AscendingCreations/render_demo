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
        pass.set_bind_group(1, &self.sprite_texture.bind_group, &[]);
        pass.set_vertex_buffer(
            0,
            self.sprite_buffer
                .vertex_buffer
                .slice(..self.sprite_buffer.vertex_count),
        );
        pass.set_index_buffer(
            self.sprite_buffer
                .indice_buffer
                .slice(..(self.sprite_buffer.indice_count * 4) as u64),
            wgpu::IndexFormat::Uint32,
        );
        pass.set_pipeline(self.sprite_pipeline.render_pipeline());
        pass.draw_indexed(0..self.sprite_buffer.indice_count as u32, 0, 0..1);
    }
}
