use crate::AscendingError;
use async_trait::async_trait;
use std::path::Path;
use wgpu::TextureFormat;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

///Handles the Device and Queue returned from WGPU.
pub struct GpuDevice {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuDevice {
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

///Handles the Window, Adapter and Surface information.
pub struct GpuWindow {
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) surface: wgpu::Surface,
    pub(crate) window: Window,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) size: PhysicalSize<f32>,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl GpuWindow {
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn resize(
        &mut self,
        gpu_device: &GpuDevice,
        size: PhysicalSize<u32>,
    ) -> Result<(), AscendingError> {
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }

        self.surface_config.height = size.height;
        self.surface_config.width = size.width;
        self.surface
            .configure(gpu_device.device(), &self.surface_config);
        self.surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        self.size = PhysicalSize::new(size.width as f32, size.height as f32);

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<f32> {
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
        gpu_device: &GpuDevice,
        event: &Event<()>,
    ) -> Result<Option<wgpu::SurfaceTexture>, AscendingError> {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == self.window.id() => match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(gpu_device, *physical_size)?;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(gpu_device, **new_inner_size)?;
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                match self.surface.get_current_texture() {
                    Ok(frame) => return Ok(Some(frame)),
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = PhysicalSize::new(
                            self.size.width as u32,
                            self.size.height as u32,
                        );
                        self.resize(gpu_device, size)?;
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

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn create_depth_texture(
        &self,
        gpu_device: &GpuDevice,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: self.size.width as u32,
            height: self.size.height as u32,
            depth_or_array_layers: 1,
        };

        let texture =
            gpu_device
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("depth texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[TextureFormat::Depth32Float],
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
    ) -> Result<(GpuWindow, GpuDevice), AscendingError>;
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
    ) -> Result<(GpuWindow, GpuDevice), AscendingError> {
        let size = window.inner_size();

        let (device, queue) =
            self.request_device(device_descriptor, trace_path).await?;

        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let caps = surface.get_capabilities(&self);

        println!("{:?}", caps.formats);
        if !caps.formats.contains(&TextureFormat::Bgra8UnormSrgb) {
            panic!("Your Rendering Device does not support Bgra8UnormSrgb")
        }

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
        };

        surface.configure(&device, &surface_config);

        Ok((
            GpuWindow {
                adapter: self,
                surface,
                window,
                surface_format: wgpu::TextureFormat::Bgra8UnormSrgb,
                size: PhysicalSize::new(size.width as f32, size.height as f32),
                surface_config,
            },
            GpuDevice { device, queue },
        ))
    }
}

#[async_trait]
pub trait InstanceExt {
    async fn create_device(
        &self,
        window: Window,
        request_adapter_options: &wgpu::RequestAdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<(GpuWindow, GpuDevice), AscendingError>;
}

#[async_trait]
impl InstanceExt for wgpu::Instance {
    async fn create_device(
        &self,
        window: Window,
        request_adapter_options: &wgpu::RequestAdapterOptions,
        device_descriptor: &wgpu::DeviceDescriptor,
        trace_path: Option<&Path>,
        present_mode: wgpu::PresentMode,
    ) -> Result<(GpuWindow, GpuDevice), AscendingError> {
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
