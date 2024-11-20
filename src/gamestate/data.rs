use cosmic_text::{CacheKey, FontSystem};
use graphics::*;
use std::collections::HashMap;
use winit::event::MouseButton;

pub struct State<Controls>
where
    Controls: camera::controls::Controls,
{
    /// World Camera Controls and time. Deturmines how the world is looked at.
    pub system: System<Controls>,
    /// Data stores for render types
    pub sprites: Vec<Image>,
    pub lights: Lights,
    pub animation: Image,
    pub map: Map,
    pub mesh: [Mesh2D; 3],
    pub rect: Rect,
    /// Atlas Groups for Textures in GPU
    pub image_atlas: AtlasSet,
    pub ui_atlas: AtlasSet,
    pub map_atlas: AtlasSet,
    pub text_atlas: TextAtlas,
    pub mesh_atlas: AtlasSet,
    /// Rendering Buffers and other shared data.
    pub ui_renderer: RectRenderer,
    pub text_renderer: TextRenderer,
    pub sprite_renderer: ImageRenderer,
    pub map_renderer: MapRenderer,
    pub light_renderer: LightRenderer,
    pub mesh_renderer: Mesh2DRenderer,
}

impl<Controls> Pass for State<Controls>
where
    Controls: camera::controls::Controls,
{
    fn render(
        &mut self,
        renderer: &GpuRenderer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: renderer.frame_buffer().as_ref().expect("no frame view?"),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.25,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: renderer.depth_buffer(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                },
            ),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Lets set the System's Shader information here, mostly Camera, Size and Time
        pass.set_bind_group(0, self.system.bind_group(), &[]);
        // Lets set the Reusable Vertices and Indicies here.
        // This is used for each Renderer, Should be more performant since it is shared.
        pass.set_vertex_buffer(0, renderer.buffer_object.vertices());
        pass.set_index_buffer(
            renderer.buffer_object.indices(),
            wgpu::IndexFormat::Uint32,
        );

        pass.render_map(renderer, &self.map_renderer, &self.map_atlas, 0);

        pass.render_image(
            renderer,
            &self.sprite_renderer,
            &self.image_atlas,
            &self.system,
            0,
        );

        pass.render_map(renderer, &self.map_renderer, &self.map_atlas, 1);
        pass.render_lights(renderer, &self.light_renderer, 0);

        pass.render_2dmeshs(renderer, &self.mesh_renderer, &self.system, 0);
        pass.render_text(renderer, &self.text_renderer, &self.text_atlas, 0);

        pass.render_rects(
            renderer,
            &self.ui_renderer,
            &self.ui_atlas,
            &self.system,
            0,
        );
    }
}
