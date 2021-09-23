use async_trait::async_trait;
use std::path::Path;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

use super::RendererError;

pub struct Renderer {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    window: Window,
    surface_format: wgpu::TextureFormat,
    size: PhysicalSize<u32>,
    present_mode: wgpu::PresentMode,
}

impl Renderer {
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.present_mode
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), RendererError> {
        let surface_format = self.surface.get_preferred_format(&self.adapter).unwrap();

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: self.present_mode,
            },
        );

        self.surface_format = surface_format;
        self.size = size;

        Ok(())
    }

    pub fn update(
        &mut self,
        event: &Event<()>,
    ) -> Result<Option<wgpu::SurfaceTexture>, RendererError> {
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
            Event::RedrawRequested(_) => match self.surface.get_current_frame() {
                Ok(frame) => return Ok(Some(frame.output)),
                Err(wgpu::SurfaceError::Lost) => {
                    self.resize(self.size)?;
                }
                Err(e) => Err(e)?,
            },
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => (),
        }

        Ok(None)
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
    ) -> Result<Renderer, RendererError>;
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
    ) -> Result<Renderer, RendererError> {
        let size = window.inner_size();

        let (device, queue) = self.request_device(device_descriptor, trace_path).await?;

        let surface = unsafe { instance.create_surface(&window) };
        let surface_format = surface.get_preferred_format(&self).unwrap();

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode,
            },
        );

        Ok(Renderer {
            adapter: self,
            device,
            queue,
            surface,
            window,
            surface_format,
            size,
            present_mode,
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
    ) -> Result<Renderer, RendererError>;
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
    ) -> Result<Renderer, RendererError> {
        let adapter = self.request_adapter(request_adapter_options).await.unwrap();
        adapter
            .create_renderer(self, window, device_descriptor, trace_path, present_mode)
            .await
    }
}
