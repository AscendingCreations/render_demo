use crate::graphics::{
    BufferLayout, CameraLayout, LayoutStorage, RendererError, SpriteVertex, TextureLayout,
    TimeLayout,
};

pub struct SpriteRenderPipeline {
    render_pipeline: wgpu::RenderPipeline,
}

impl SpriteRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        layout_storage: &mut LayoutStorage,
    ) -> Result<Self, RendererError> {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/spriteshader.wgsl").into(),
            ),
        });

        let camera_layout = layout_storage.create_layout(device, CameraLayout);
        let texture_layout = layout_storage.create_layout(device, TextureLayout);
        let time_layout = layout_storage.create_layout(device, TimeLayout);

        // Set up the pipeline layout.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[&camera_layout, &time_layout, &texture_layout],
                push_constant_ranges: &[],
            });

        // Create the render pipeline.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: SpriteVertex::stride(),
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &SpriteVertex::attributes(),
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.,
                    clamp: 0.,
                },
            }),
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                ..Default::default()
            },
        });

        Ok(Self { render_pipeline })
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }
}
