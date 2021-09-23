use bytemuck::{Pod, Zeroable};
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use ultraviolet::Mat4;
use wgpu::util::DeviceExt;

use super::{
    CameraLayout, Layout, LayoutStorage, RendererError, Sprite, Transform, VertexFormat,
    VertexFormatExt,
};

pub struct SpriteRenderPipeline {
    render_pipeline: wgpu::RenderPipeline,
}

impl SpriteRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        layout_storage: &mut LayoutStorage,
        vertex_format: VertexFormat,
    ) -> Result<Self, Error> {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_layout = layout_storage.create_layout(device, CameraLayout);
        let sprite_layout = layout_storage.create_layout(device, SpriteLayout);

        // Set up the pipeline layout.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[&camera_layout, &sprite_layout],
                push_constant_ranges: &[],
            });

        // Create the render pipeline.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_format.stride() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_format.attributes(),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Ok(Self { render_pipeline })
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
}

pub trait RenderSprite<'a, 'b>
where
    'b: 'a,
{
    fn render_sprite(&mut self, sprite: &'b Sprite, pipeline: &'b SpriteRenderPipeline);
}

impl<'a, 'b> RenderSprite<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_sprite(&mut self, sprite: &'b Sprite, pipeline: &'b SpriteRenderPipeline) {
        let model = &instance.model;

        for (transform_index, mesh_index) in model.instances() {
            let mesh = &model.meshes()[*mesh_index];

            self.set_bind_group(
                2,
                instance.transform_bind_group(),
                &[instance.map_transform_offset(*transform_index)],
            );

            for primitive in mesh.primitives() {
                let vertex_format = primitive.vertex_format();

                let material = &model.materials()[primitive.material()];
                let (type_id, bytes) = material.layout_key();

                let render_pipeline =
                    match model_renderer
                        .render_pipelines
                        .get(&(vertex_format, type_id, bytes))
                    {
                        Some(render_pipeline) => render_pipeline,
                        _ => continue,
                    };

                self.set_pipeline(render_pipeline.render_pipeline());
                self.set_bind_group(3, material.bind_group(), &[]);
                self.set_vertex_buffer(0, primitive.vertex_buffer().slice(..));
                self.set_index_buffer(
                    primitive.index_buffer().unwrap().slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                self.draw_indexed(0..(primitive.index_count() as u32), 0, 0..1);
            }
        }
    }
}
