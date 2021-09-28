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
    }
}
