use crate::graphics::*;
use cosmic_text::{CacheKey, FontSystem};
use std::collections::HashMap;
use wgpu_profiler::{wgpu_profiler, GpuProfiler};

pub struct State<Controls>
where
    Controls: camera::controls::Controls,
{
    /// Storage container for layouts for faster initlization
    pub layout_storage: LayoutStorage,
    /// World Camera Controls and time. Deturmines how the world is looked at.
    pub system: System<Controls>,
    /// Sprite data TODO: Make an array,
    pub sprites: Vec<Sprite>,
    /// Render pipe line for Sprites
    pub sprite_pipeline: SpriteRenderPipeline,
    /// Vertex buffer group for Sprites
    pub sprite_buffer: InstanceBuffer<SpriteVertex>,
    /// AtlasGroup to hold Sprite Images
    pub sprite_atlas: AtlasGroup,
    /// maps TODO: make this an array.
    pub map: Map,
    /// Render Pipeline for maps
    pub map_pipeline: MapRenderPipeline,
    /// vertex buffer group for maps
    pub maplower_buffer: InstanceBuffer<MapVertex>,
    pub mapupper_buffer: InstanceBuffer<MapVertex>,
    /// Texture Bind group for Maptextures
    pub map_group: TextureGroup,
    /// contains the Map layer grids in pixel form.
    pub map_textures: MapTextures,
    /// contains the Tile images.
    pub map_atlas: AtlasGroup,
    /// animation test stuff.
    pub animation: Sprite,
    pub animation_buffer: InstanceBuffer<SpriteVertex>,
    pub animation_atlas: AtlasGroup,

    /// Basic shape rendering.
    pub shapes: Shapes,
    pub shapes_buffer: GpuBuffer<ShapeVertex>,
    pub shapes_pipeline: ShapeRenderPipeline,

    /// Text test stuff.
    pub text: Text,
    pub text_buffer: InstanceBuffer<TextVertex>,
    pub text_pipeline: TextRenderPipeline,
    pub text_atlas: AtlasGroup<CacheKey>,
    pub emoji_atlas: AtlasGroup<CacheKey>,
    pub profiler: GpuProfiler,
    pub buffer_object: StaticBufferObject,
    pub buffer_object1: StaticBufferObject,
    pub buffer_object2: StaticBufferObject,
    pub buffer_object3: StaticBufferObject,
    pub buffer_object4: StaticBufferObject,
}

impl<Controls> Pass for State<Controls>
where
    Controls: camera::controls::Controls,
{
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        views: &HashMap<String, wgpu::TextureView>,
        renderer: &crate::Renderer,
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

        pass.set_bind_group(0, self.system.bind_group(), &[]);
        pass.set_vertex_buffer(0, self.buffer_object.vertices());
        pass.set_index_buffer(
            self.buffer_object.indices(),
            wgpu::IndexFormat::Uint16,
        );

        pass.render_maps(
            &self.maplower_buffer,
            &self.map_atlas,
            &self.map_group,
            &self.map_pipeline,
        );

        pass.set_vertex_buffer(0, self.buffer_object1.vertices());
        pass.set_index_buffer(
            self.buffer_object1.indices(),
            wgpu::IndexFormat::Uint16,
        );
        pass.render_sprite(
            &self.sprite_buffer,
            &self.sprite_atlas,
            &self.sprite_pipeline,
        );

        pass.set_vertex_buffer(0, self.buffer_object3.vertices());
        pass.set_index_buffer(
            self.buffer_object3.indices(),
            wgpu::IndexFormat::Uint16,
        );
        pass.render_sprite(
            &self.animation_buffer,
            &self.animation_atlas,
            &self.sprite_pipeline,
        );

        pass.set_vertex_buffer(0, self.buffer_object.vertices());
        pass.set_index_buffer(
            self.buffer_object.indices(),
            wgpu::IndexFormat::Uint16,
        );
        pass.render_maps(
            &self.mapupper_buffer,
            &self.map_atlas,
            &self.map_group,
            &self.map_pipeline,
        );

        pass.set_vertex_buffer(0, self.buffer_object2.vertices());
        pass.set_index_buffer(
            self.buffer_object.indices(),
            wgpu::IndexFormat::Uint16,
        );
        self.profiler
            .begin_scope("Text", &mut pass, &renderer.device());
        pass.render_text(
            &self.text_buffer,
            &self.text_atlas,
            &self.emoji_atlas,
            &self.text_pipeline,
        );
        self.profiler.end_scope(&mut pass);

        //pass.render_shape(&self.shapes_buffer, &self.shapes_pipeline);
    }
}
