use crate::AscendingError;
use async_trait::async_trait;
use std::path::Path;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

pub struct Renderer {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    window: Window,
    surface_format: wgpu::TextureFormat,
    size: PhysicalSize<u32>,
    present_mode: wgpu::PresentMode,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.present_mode
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<(), AscendingError> {
        let surface_format =
            self.surface.get_supported_formats(&self.adapter)[0];

        if size.width == 0 || size.height == 0 {
            return Ok(());
        }

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: self.present_mode,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
            },
        );

        self.surface_format = surface_format;
        self.size = size;

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn update(
        &mut self,
        event: &Event<()>,
    ) -> Result<Option<wgpu::SurfaceTexture>, AscendingError> {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == self.window.id() => match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(*physical_size)?;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(**new_inner_size)?;
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                match self.surface.get_current_texture() {
                    Ok(frame) => return Ok(Some(frame)),
                    Err(wgpu::SurfaceError::Lost) => {
                        self.resize(self.size)?;
                    }
                    Err(wgpu::SurfaceError::Outdated) => {
                        return Ok(None);
                    }
                    Err(e) => return Err(AscendingError::from(e)),
                }
            }
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => (),
        }

        Ok(None)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn create_depth_texture(&self) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: self.size.width,
            height: self.size.height,
            depth_or_array_layers: 1,
        };

        let texture = self.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
        });

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

#[async_trait]
pub trait AdapterExt {
    async fn create_renderer(
        self,
        instance: &wgpu::Instance,
        window: Window,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<Renderer, AscendingError>;
}

#[async_trait]
impl AdapterExt for wgpu::Adapter {
    async fn create_renderer(
        self,
        instance: &wgpu::Instance,
        window: Window,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<Renderer, AscendingError> {
        let size = window.inner_size();

        let (device, queue) =
            self.request_device(device_descriptor, trace_path).await?;

        let surface = unsafe { instance.create_surface(&window) };
        let surface_format = surface.get_supported_formats(&self)[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &surface_config);

        Ok(Renderer {
            adapter: self,
            device,
            queue,
            surface,
            window,
            surface_format,
            size,
            present_mode,
            surface_config,
        })
    }
}

#[async_trait]
pub trait InstanceExt {
    async fn create_renderer(
        &self,
        window: Window,
        request_adapter_options: &wgpu::RequestAdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<Renderer, AscendingError>;
}

#[async_trait]
impl InstanceExt for wgpu::Instance {
    async fn create_renderer(
        &self,
        window: Window,
        request_adapter_options: &wgpu::RequestAdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<Renderer, AscendingError> {
        let adapter =
            self.request_adapter(request_adapter_options).await.unwrap();
        adapter
            .create_renderer(
                self,
                window,
                device_descriptor,
                trace_path,
                present_mode,
            )
            .await
    }
}
