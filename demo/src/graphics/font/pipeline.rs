use crate::{
    graphics::{
        BufferLayout, LayoutStorage, SystemLayout, TextVertex, TextureLayout,
    },
    AscendingError,
};

pub struct TextRenderPipeline {
    render_pipeline: wgpu::RenderPipeline,
}

impl TextRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        layout_storage: &mut LayoutStorage,
    ) -> Result<Self, AscendingError> {
        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/textshader.wgsl").into(),
                ),
            });

        let system_layout = layout_storage.create_layout(device, SystemLayout);
        let texture_layout =
            layout_storage.create_layout(device, TextureLayout);

        // Create the render pipeline.
        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Text render pipeline"),
                layout: Some(&device.create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: Some("Text_render_pipeline_layout"),
                        bind_group_layouts: &[&system_layout, &texture_layout],
                        push_constant_ranges: &[],
                    },
                )),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: TextVertex::vertex_stride() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &TextVertex::attributes(),
                    }],
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
                        blend: Some(
                            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        ),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Ok(Self { render_pipeline })
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
}
