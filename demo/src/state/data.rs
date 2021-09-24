use crate::graphics::{
    Atlas, Camera, LayoutStorage, Pass, RenderSprite, Sprite, SpriteBuffer, SpriteRenderPipeline,
    TextureGroup,
};
use std::collections::HashMap;

pub struct State<Controls>
where
    Controls: camera::controls::Controls,
{
    pub sprite: Sprite,
    pub sprite_pipeline: SpriteRenderPipeline,
    pub sprite_buffer: SpriteBuffer,
    pub sprite_atlas: Atlas,
    pub layout_storage: LayoutStorage,
    pub sprite_texture: TextureGroup,
    pub camera: Camera<Controls>,
}

impl<Controls> Pass for State<Controls>
where
    Controls: camera::controls::Controls,
{
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        views: &HashMap<String, wgpu::TextureView>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: views.get("framebuffer").as_ref().expect("no frame view?"),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.25,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: views.get("depthbuffer").as_ref().expect("no depth view?"),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        pass.set_bind_group(0, self.camera.bind_group(), &[]);
        pass.render_sprite(
            &self.sprite_buffer,
            &self.sprite_texture,
            &self.sprite_pipeline,
        );
    }
}
