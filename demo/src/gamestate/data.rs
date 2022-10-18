pub(crate) use crate::graphics::*;
use fontdue::layout::GlyphRasterConfig;
use fontdue::{Font, FontSettings};
use std::collections::HashMap;
pub struct State<Controls>
where
    Controls: camera::controls::Controls,
{
    /// Storage container for layouts for faster initlization
    pub layout_storage: LayoutStorage,
    /// World Camera Controls. Deturmines how the world is looked at.
    pub camera: Camera<Controls>,
    /// time for all animation on shader side.
    pub time_group: TimeGroup,
    /// Screen Size to the shaders.
    pub screen_group: ScreenGroup,
    /// Sprite data TODO: Make an array,
    pub sprite: [Sprite; 2],
    /// Render pipe line for Sprites
    pub sprite_pipeline: SpriteRenderPipeline,
    /// Vertex buffer group for Sprites
    pub sprite_buffer: GpuBuffer<SpriteVertex>,
    /// Atlas to hold Sprite Images
    pub sprite_atlas: Atlas,
    /// Texture Bind group for Sprite Atlas
    pub sprite_texture: TextureGroup,
    /// maps TODO: make this an array.
    pub map: Map,
    /// Render Pipeline for maps
    pub map_pipeline: MapRenderPipeline,
    /// vertex buffer group for maps
    pub maplower_buffer: GpuBuffer<MapVertex>,
    pub mapupper_buffer: GpuBuffer<MapVertex>,
    /// Texture bind group for Map Atlas
    pub map_texture: TextureGroup,
    /// Texture Bind group for Maptextures
    pub map_group: TextureGroup,
    /// contains the Tile images.
    pub map_atlas: Atlas,
    /// contains the Map layer grids in pixel form.
    pub map_textures: MapTextures,

    /// animation test stuff.
    pub animation: Animation,
    pub animation_buffer: GpuBuffer<AnimationVertex>,
    pub animation_pipeline: AnimationRenderPipeline,
    pub animation_atlas: Atlas,
    pub animation_texture: TextureGroup,

    /// Basic shape rendering.
    pub shapes: Shape,
    pub shapes_buffer: GpuBuffer<ShapeVertex>,
    pub shapes_pipeline: ShapeRenderPipeline,

    /// Text test stuff.
    pub text: Text,
    pub text_buffer: GpuBuffer<TextVertex>,
    pub text_pipeline: TextRenderPipeline,
    pub text_atlas: Atlas<GlyphRasterConfig>,
    pub text_texture: TextureGroup,
    pub fonts: Vec<Font>,
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
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: views
                    .get("framebuffer")
                    .as_ref()
                    .expect("no frame view?"),
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
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: views
                        .get("depthbuffer")
                        .as_ref()
                        .expect("no depth view?"),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                },
            ),
        });

        pass.set_bind_group(0, self.camera.bind_group(), &[]);
        pass.set_bind_group(1, self.time_group.bind_group(), &[]);

        pass.render_maps(
            &self.maplower_buffer,
            &self.map_texture,
            &self.map_group,
            &self.map_pipeline,
        );
        pass.render_sprite(
            &self.sprite_buffer,
            &self.sprite_texture,
            &self.sprite_pipeline,
            &self.screen_group,
        );

        pass.render_text(
            &self.text_buffer,
            &self.text_texture,
            &self.text_pipeline,
            &self.screen_group,
        );

        pass.render_animations(
            &self.animation_buffer,
            &self.animation_texture,
            &self.animation_pipeline,
        );

        pass.render_maps(
            &self.mapupper_buffer,
            &self.map_texture,
            &self.map_group,
            &self.map_pipeline,
        );

        pass.render_shape(&self.shapes_buffer, &self.shapes_pipeline);
    }
}
