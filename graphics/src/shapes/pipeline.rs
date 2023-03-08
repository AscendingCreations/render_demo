use crate::{
    AscendingError, GpuRenderer, InstanceLayout, RectVertex,
    StaticBufferObject, SystemLayout, TextureLayout,
};

pub struct RectsRenderPipeline {
    render_pipeline: wgpu::RenderPipeline,
}

impl RectsRenderPipeline {
    pub fn new(
        renderer: &mut GpuRenderer,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, AscendingError> {
        let shader = renderer.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/guishader.wgsl").into(),
                ),
            },
        );

        let system_layout = renderer.create_layout(SystemLayout);
        let texture_layout = renderer.create_layout(TextureLayout);

        // Create the render pipeline.
        let render_pipeline = renderer.device().create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Shape render pipeline"),
                layout: Some(&renderer.device().create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: Some("render_pipeline_layout"),
                        bind_group_layouts: &[&system_layout, &texture_layout],
                        push_constant_ranges: &[],
                    },
                )),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: StaticBufferObject::vertex_size(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                StaticBufferObject::vertex_attribute(),
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: RectVertex::instance_stride() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &RectVertex::attributes(),
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fragment",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            },
        );

        Ok(Self { render_pipeline })
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
}
