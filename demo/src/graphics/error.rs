use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Surface(#[from] wgpu::SurfaceError),
    #[error(transparent)]
    WGpu(#[from] wgpu::Error),
    #[error(transparent)]
    Device(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}
