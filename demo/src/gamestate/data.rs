use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;

use crate::gui::RenderWidgets;

pub struct State<Controls>
where
    Controls: camera::controls::Controls,
{
    /// World Camera Controls and time. Deturmines how the world is looked at.
    pub system: System<Controls>,
    /// Sprite data TODO: Make an array,
    pub sprites: Vec<Image>,
    pub animation: Image,
    /// Render pipe line for Sprites
    pub sprite_renderer: ImageRenderer,
    pub animation_renderer: ImageRenderer,
    /// AtlasGroup to hold Sprite/animation Images
    pub image_atlas: AtlasGroup,
    /// maps TODO: make this an array.
    pub map: Map,
    /// vertex buffer group for maps
    pub map_renderer: MapRenderer,
    /// contains the Tile images.
    pub map_atlas: AtlasGroup,
    /// Basic shape rendering.
    pub rects: Rect,
    pub rects_renderer: RectRenderer,
    pub rects_atlas: AtlasGroup,
    /// Text test stuff.
    pub text_atlas: TextAtlas,
    pub text_renderer: TextRenderer,
}

impl<Controls> Pass<crate::UIBuffer> for State<Controls>
where
    Controls: camera::controls::Controls,
{
    fn render(
        &mut self,
        renderer: &GpuRenderer,
        encoder: &mut wgpu::CommandEncoder,
        views: &HashMap<String, wgpu::TextureView>,
        ui_buffer: &crate::UIBuffer,
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

        // Lets set the System's Shader information here, mostly Camera, Size and Time
        pass.set_bind_group(0, self.system.bind_group(), &[]);

        // Lets set the Reusable Vertices and Indicies here.
        // This is used for each Renderer, Should be more performant since it is shared.
        pass.set_vertex_buffer(0, renderer.buffer_object.vertices());
        pass.set_index_buffer(
            renderer.buffer_object.indices(),
            wgpu::IndexFormat::Uint16,
        );

        pass.render_lower_maps(renderer, &self.map_renderer, &self.map_atlas);

        pass.render_image(renderer, &self.sprite_renderer, &self.image_atlas);

        pass.render_image(
            renderer,
            &self.animation_renderer,
            &self.image_atlas,
        );

        pass.render_upper_maps(renderer, &self.map_renderer, &self.map_atlas);

        pass.render_text(renderer, &self.text_renderer, &self.text_atlas);

        pass.render_rects(
            renderer,
            &self.rects_renderer,
            &self.rects_atlas,
            &self.system,
        );

        pass.render_widgets(renderer, ui_buffer, &self.system);
    }
}
