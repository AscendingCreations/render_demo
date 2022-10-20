use crate::gamestate::group::AtlasGroup;
pub(crate) use crate::graphics::{
    atlas::{Allocation, Atlas},
    RendererError,
};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::{
    io::{Error, ErrorKind},
    path::Path,
};

#[derive(Clone, Debug)]
pub struct Texture {
    name: String,
    bytes: Vec<u8>,
    size: (u32, u32),
}

impl Texture {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, RendererError> {
        let name = path
            .as_ref()
            .file_name()
            .ok_or_else(|| {
                Error::new(ErrorKind::Other, "could not get filename")
            })?
            .to_os_string()
            .into_string()
            .map_err(|_| {
                Error::new(ErrorKind::Other, "could not convert name to String")
            })?;

        Ok(Self::from_image(name, image::open(path)?))
    }

    pub fn from_image(name: String, image: DynamicImage) -> Self {
        let size = image.dimensions();
        let bytes = image.into_rgba8().into_raw();

        Self { name, bytes, size }
    }

    pub fn from_memory(
        name: String,
        data: &[u8],
    ) -> Result<Self, RendererError> {
        Ok(Self::from_image(name, image::load_from_memory(data)?))
    }

    pub fn from_memory_with_format(
        name: String,
        data: &[u8],
        format: ImageFormat,
    ) -> Result<Self, RendererError> {
        Ok(Self::from_image(
            name,
            image::load_from_memory_with_format(data, format)?,
        ))
    }

    pub fn upload(
        &self,
        atlas: &mut Atlas,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Allocation> {
        let (width, height) = self.size;
        atlas.upload(
            self.name.clone(),
            &self.bytes,
            width,
            height,
            device,
            queue,
        )
    }

    pub fn group_upload(
        &self,
        atlas_group: &mut AtlasGroup,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Allocation> {
        let (width, height) = self.size;
        atlas_group.atlas.upload(
            self.name.clone(),
            &self.bytes,
            width,
            height,
            device,
            queue,
        )
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}
